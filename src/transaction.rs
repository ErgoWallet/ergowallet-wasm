use wasm_bindgen::prelude::*;
use std::convert::{TryFrom, TryInto};

use k256::Scalar;
use sigma_tree::chain::{Base16DecodedBytes, Digest32};
use sigma_tree::sigma_protocol::prover::TestProver;
use sigma_tree::sigma_protocol::{DlogProverInput, PrivateInput};
use sigma_tree::sigma_protocol::GroupSizedBytes;
use sigma_tree::chain::transaction::unsigned::UnsignedTransaction;
use sigma_tree::chain::input::{UnsignedInput, Input};
use sigma_tree::chain::context_extension::ContextExtension;
use sigma_tree::chain::ergo_box::{ErgoBox, ErgoBoxCandidate};
use sigma_tree::wallet::signing::sign_transaction;
use sigma_tree::chain::ergo_state_context::ErgoStateContext;
use sigma_tree::chain::address::{AddressEncoder, NetworkPrefix};
use sigma_tree::chain::ergo_box::box_id::BoxId;
use sigma_tree::chain::prover_result::{ProverResult, ProofBytes};
use sigma_tree::chain::contract::Contract;
use sigma_tree::chain::token::{TokenAmount, TokenId};
use sigma_tree::chain::ergo_box::box_value::BoxValue;
use sigma_tree::chain::ergo_box::register::NonMandatoryRegisters;
use crate::MINER_ERGO_TREE;
use sigma_tree::{ErgoTree, chain};
use sigma_tree::serialization::SigmaSerializable;

#[derive(Serialize, Deserialize)]
pub struct AssetValue {
    #[serde(rename = "tokenId")]
    pub token_id: String,
    pub amount: String
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

#[wasm_bindgen]
pub struct Transaction(chain::transaction::Transaction);

#[wasm_bindgen]
impl Transaction {
    /// JSON representation
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        JsValue::from_serde(&self.0).map_err(|e| JsValue::from_str(&format!("{}", e)))
    }
}

impl From<chain::transaction::Transaction> for Transaction {
    fn from(t: chain::transaction::Transaction) -> Self {
        Transaction(t)
    }
}

fn fee_box_candidate(fee_amount: u64, creation_height: u32) -> ErgoBoxCandidate {
    let ergo_tree_bytes = Base16DecodedBytes::try_from(MINER_ERGO_TREE.to_string()).unwrap();
    let fee_ergo_tree = ErgoTree::sigma_parse_bytes(ergo_tree_bytes.0).unwrap();
    ErgoBoxCandidate {
        value: BoxValue::new(fee_amount).unwrap(),
        ergo_tree: fee_ergo_tree,
        tokens: vec![],
        additional_registers: NonMandatoryRegisters::empty(),
        creation_height,
    }
}

#[wasm_bindgen]
pub fn create_tx(
    inputs: Box<[JsValue]>,
    outputs: Box<[JsValue]>,
    fee_amount: u64,
    height: u32,
) -> Result<JsValue, JsValue> {
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
    let _inputs: Vec<Input> = inputs_from_js
        .iter()
        .map(|x| {
            let id_bytes = Base16DecodedBytes::try_from(x.box_id.clone()).unwrap();
            let digest = Digest32::try_from(id_bytes).unwrap();
            let box_id: BoxId = BoxId(digest);

            Input {
                box_id,
                spending_proof: ProverResult {
                    proof: ProofBytes::Empty,
                    extension: ContextExtension::empty(),
                },
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
            let tokens = x.assets.iter().map(|t| {
                let id_bytes = Base16DecodedBytes::try_from(t.token_id.clone()).unwrap();
                let digest = Digest32::try_from(id_bytes).unwrap();
                TokenAmount {
                    token_id: TokenId(digest),
                    amount: t.amount.parse::<u64>().unwrap()
                }
            }).collect();

            ErgoBoxCandidate {
                value: BoxValue::new(val).unwrap(),
                ergo_tree: contract.get_ergo_tree(),
                tokens,
                additional_registers: NonMandatoryRegisters::empty(),
                creation_height: height,
            }
        })
        .collect();

    // add one output for miner fee
    _outputs.push(fee_box_candidate(fee_amount, height));

    // create transaction
    let tx = chain::transaction::Transaction::new(_inputs, vec![], _outputs);

    Ok(JsValue::from_serde(&tx).unwrap())
}


#[wasm_bindgen]
pub fn sign_tx(
    secret_keys: Box<[JsValue]>,
    boxes_to_spend: Box<[JsValue]>,
    tx: &JsValue
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
                PrivateInput::DlogProverInput(
                    DlogProverInput {
                        w: Scalar::from(GroupSizedBytes::from(bytes))
                    })
            })
            .collect(),
    };

    // 2. Construct unsigned transaction
    let tx: chain::transaction::Transaction = tx.into_serde().unwrap();
    let unsigned_inputs = tx.inputs.iter().map(|i| UnsignedInput {
        box_id: i.box_id.clone(),
        extension: ContextExtension::empty()
    }).collect();

    let unsigned = UnsignedTransaction::new(
        unsigned_inputs,
        tx.data_inputs,
        tx.output_candidates,
    );

    let res = sign_transaction(
        &prover,
        unsigned,
        boxes_to_spend.as_slice(),
        vec![].as_slice(),
        &ErgoStateContext::dummy()
    ).map_err(|e| JsValue::from_str(&format!("{}", e)))
     .map(Transaction::from);

    res
}