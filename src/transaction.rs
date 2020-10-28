use wasm_bindgen::prelude::*;

use std::convert::{TryFrom, TryInto};

use ergo_lib::chain::address::{AddressEncoder, NetworkPrefix};
use ergo_lib::chain::context_extension::ContextExtension;
use ergo_lib::chain::contract::Contract;
use ergo_lib::chain::ergo_box::box_id::BoxId;
use ergo_lib::chain::ergo_box::box_value::BoxValue;
use ergo_lib::chain::ergo_box::register::NonMandatoryRegisters;
use ergo_lib::chain::ergo_box::{ErgoBox, ErgoBoxCandidate};
use ergo_lib::chain::ergo_state_context::ErgoStateContext;
use ergo_lib::chain::input::{Input, UnsignedInput};
use ergo_lib::chain::prover_result::{ProofBytes, ProverResult};
use ergo_lib::chain::token::{TokenAmount, TokenId, Token};
use ergo_lib::serialization::SigmaSerializable;
use ergo_lib::sigma_protocol::prover::TestProver;
use ergo_lib::sigma_protocol::{DlogProverInput, PrivateInput};
use ergo_lib::wallet::signing::sign_transaction;
use ergo_lib::{chain, ErgoTree};
use ergo_lib::chain::Digest32;
use ergo_lib::chain::Base16DecodedBytes;

use k256::Scalar;
// use elliptic_curve::FromBytes;

use crate::{MINER_ERGO_TREE, MINERS_FEE_MAINNET_ADDRESS};


#[derive(Serialize, Deserialize)]
pub struct AssetValue {
    #[serde(rename = "tokenId")]
    pub token_id: String,
    pub amount: String,
}

#[derive(Serialize, Deserialize)]
pub struct TxInput {
    #[serde(rename = "boxId")]
    pub box_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct TxOutput {
    pub value: String,
    pub address: String,
    pub assets: Vec<AssetValue>,
}


/// Unsigned (inputs without proofs) transaction
#[wasm_bindgen]
#[derive(PartialEq, Debug, Clone)]
pub struct UnsignedTransaction(chain::transaction::unsigned::UnsignedTransaction);

#[wasm_bindgen]
impl UnsignedTransaction {
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        JsValue::from_serde(&self.0).map_err(|e| JsValue::from_str(&format!("{}", e)))
    }
}

impl From<chain::transaction::unsigned::UnsignedTransaction> for UnsignedTransaction {
    fn from(t: chain::transaction::unsigned::UnsignedTransaction) -> Self {
        UnsignedTransaction(t)
    }
}

impl From<UnsignedTransaction> for chain::transaction::unsigned::UnsignedTransaction {
    fn from(t: UnsignedTransaction) -> Self {
        t.0
    }
}

#[wasm_bindgen]
pub struct Transaction(chain::transaction::Transaction);

impl From<chain::transaction::Transaction> for Transaction {
    fn from(t: chain::transaction::Transaction) -> Self {
        Transaction(t)
    }
}

#[wasm_bindgen]
impl Transaction {
    /// JSON representation
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        JsValue::from_serde(&self.0).map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    #[wasm_bindgen]
    pub fn create(
        inputs: Box<[JsValue]>,
        outputs: Box<[JsValue]>,
        fee_amount: u64,
        height: u32,
    ) -> Result<UnsignedTransaction, JsValue> {
        let inputs_from_js: Vec<TxInput> = inputs
            .into_iter()
            .map(|x| x.into_serde().unwrap())
            .collect();

        let outputs_from_js: Vec<TxOutput> = outputs
            .into_iter()
            .map(|x| x.into_serde().unwrap())
            .collect();

        let encoder = AddressEncoder::new(NetworkPrefix::Mainnet);

        // construct inputs without proofs
        let _inputs: Vec<UnsignedInput> = inputs_from_js
            .iter()
            .map(|x| {
                let id_bytes = Base16DecodedBytes::try_from(x.box_id.clone()).unwrap();
                let digest = Digest32::try_from(id_bytes).unwrap();
                let box_id: BoxId = BoxId(digest);

                UnsignedInput {
                    box_id,
                    extension: ContextExtension::empty(),
                }
            })
            .collect();

        // construct outputs
        let mut _outputs: Vec<ErgoBoxCandidate> = outputs_from_js
            .iter()
            .map(|x| {
                let addr = encoder.parse_address_from_str(x.address.as_str()).unwrap();
                let contract = Contract::pay_to_address(&addr).unwrap();

                let val = x.value.parse::<u64>().unwrap();

                // tokens
                let tokens = x
                    .assets
                    .iter()
                    .map(|t| {
                        let id_bytes = Base16DecodedBytes::try_from(t.token_id.clone()).unwrap();
                        let digest = Digest32::try_from(id_bytes).unwrap();
                        Token {
                            token_id: TokenId(digest),
                            amount: TokenAmount::try_from(t.amount.parse::<u64>().unwrap()).unwrap(),
                        }
                    })
                    .collect();

                ErgoBoxCandidate {
                    value: BoxValue::new(val).unwrap(),
                    ergo_tree: contract.ergo_tree(),
                    tokens,
                    additional_registers: NonMandatoryRegisters::empty(),
                    creation_height: height,
                }
            })
            .collect();

        // add one output for miner fee
        _outputs.push(Self::fee_box_candidate(fee_amount, height));

        // create transaction
        let tx = chain::transaction::unsigned::UnsignedTransaction::new(_inputs, vec![], _outputs);

        Ok(UnsignedTransaction(tx))
    }

    #[wasm_bindgen]
    pub fn sign(
        secret_keys: Box<[JsValue]>,
        boxes_to_spend: Box<[JsValue]>,
        tx: &JsValue,
    ) -> Result<Transaction, JsValue> {
        let secrets: Vec<String> = secret_keys
            .into_iter()
            .map(|x| x.into_serde().unwrap())
            .collect();

        let boxes_to_spend: Vec<ErgoBox> = boxes_to_spend
            .into_iter()
            .map(|x| x.into_serde().unwrap())
            .collect();

        // 1. Construct prover from secret keys
        let prover = TestProver {
            secrets: secrets
                .into_iter()
                .map(|s| {
                    let scalar_bytes = Base16DecodedBytes::try_from(s).unwrap();
                    let bytes: &[u8; 32] = scalar_bytes.0.as_slice().try_into().unwrap();

                    PrivateInput::DlogProverInput(DlogProverInput::from_bytes(bytes).unwrap())
                })
                .collect(),
        };

        // 2. Construct unsigned transaction
        let unsigned: chain::transaction::unsigned::UnsignedTransaction = tx.into_serde().unwrap();


        let res = sign_transaction(
            &prover,
            unsigned,
            boxes_to_spend.as_slice(),
            vec![].as_slice(),
            &ErgoStateContext::dummy(),
        )
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
        .map(Transaction::from);

        res
    }

    fn fee_box_candidate(fee_amount: u64, creation_height: u32) -> ErgoBoxCandidate {
        let address_encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
        let miner_fee_address = address_encoder
            .parse_address_from_str(MINERS_FEE_MAINNET_ADDRESS)
            .unwrap();
        let fee_ergo_tree = miner_fee_address.script().unwrap();

        //let ergo_tree_bytes = Base16DecodedBytes::try_from(MINER_ERGO_TREE.to_string()).unwrap();
        //let fee_ergo_tree = ErgoTree::sigma_parse_bytes(ergo_tree_bytes.0).unwrap();
        ErgoBoxCandidate {
            value: BoxValue::new(fee_amount).unwrap(),
            ergo_tree: fee_ergo_tree,
            tokens: vec![],
            additional_registers: NonMandatoryRegisters::empty(),
            creation_height,
        }
    }
}


