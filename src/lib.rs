#[macro_use]
extern crate serde_derive;

use std::convert::TryFrom;

use hdpath::StandardHDPath;
use sigma_tree::{chain, ErgoTree};
use sigma_tree::chain::{Base16EncodedBytes, Contract, Input, TokenAmount, TokenId};
use sigma_tree::serialization::*;
use sigma_tree::serialization::serializable::*;
use sigma_tree::sigma_protocol;
use sigma_tree::sigma_protocol::sigma_boolean::ProveDlog;
use wasm_bindgen::prelude::*;
use web_sys::console;

pub use address::*;
pub use password_crypto::*;
use sigma_tree::chain::register::NonMandatoryRegisters;

mod address;
mod password_crypto;
mod utils;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const MINER_ERGO_TREE: &str = "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304";
const ABSOLUTE_MIN_GAP_LIMIT: i32 = 21;


#[wasm_bindgen]
pub struct KeyManager {}

#[wasm_bindgen]
impl KeyManager {
    pub fn recover(mnemonic: &str) -> KeyManager {
        console::log_1(&"Recovering KeyManager from mnemonic phrase".into());

        return KeyManager {};
    }
}

#[derive(Serialize, Deserialize)]
pub struct AssetValue {
    #[serde(rename = "tokenId")]
    pub token_id: String,
    pub amount: String
}

#[derive(Serialize, Deserialize)]
pub struct TxOutput {
    pub value: String,
    pub address: String,
    pub assets: Vec<AssetValue>,
}

#[derive(Serialize, Deserialize)]
pub struct TxInput {
    #[serde(rename = "boxId")]
    pub box_id: String,
}

#[wasm_bindgen]
pub fn serialize_tx(js_tx: &JsValue) -> Vec<u8> {
    let tx: chain::Transaction = js_tx.into_serde().unwrap();
    tx.sigma_serialise_bytes()
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

    let encoder = chain::AddressEncoder::new(chain::NetworkPrefix::Mainnet);

    // construct inputs without proofs
    let _inputs: Vec<chain::Input> = inputs_from_js
        .iter()
        .map(|x| {
            let id_bytes = chain::Base16DecodedBytes::try_from(x.box_id.clone()).unwrap();
            let digest = chain::Digest32::try_from(id_bytes).unwrap();
            let box_id: chain::BoxId = chain::BoxId(digest);

            chain::Input {
                box_id,
                spending_proof: chain::ProverResult {
                    proof: chain::ProofBytes::Empty,
                    extension: chain::ContextExtension::empty(),
                },
            }
        })
        .collect();

    // construct outputs
    let mut _outputs: Vec<chain::ErgoBoxCandidate> = outputs_from_js
        .iter()
        .map(|x| {
            let addr = encoder.parse_address_from_str(x.address.as_str()).unwrap();
            let contract = chain::Contract::pay_to_address(addr).unwrap();

            let val = x.value.parse::<u64>().unwrap();

            // tokens
            let tokens = x.assets.iter().map(|t| {
                let id_bytes = chain::Base16DecodedBytes::try_from(t.token_id.clone()).unwrap();
                let digest = chain::Digest32::try_from(id_bytes).unwrap();
                chain::TokenAmount {
                    token_id: TokenId(digest),
                    amount: t.amount.parse::<u64>().unwrap()
                }
            }).collect();

            chain::ErgoBoxCandidate {
                value: chain::box_value::BoxValue::new(val).unwrap(),
                ergo_tree: contract.get_ergo_tree(),
                tokens,
                additional_registers: NonMandatoryRegisters::empty(),
                creation_height: height,
            }

        })
        .collect();

    // add one output for miner fee
    let ergo_tree_bytes = chain::Base16DecodedBytes::try_from(MINER_ERGO_TREE.to_string()).unwrap();
    let fee_ergo_tree = ErgoTree::sigma_parse_bytes(ergo_tree_bytes.0).unwrap();
    let fee_box = chain::ErgoBoxCandidate::new(
        chain::box_value::BoxValue::new(fee_amount).unwrap(),
        fee_ergo_tree,
        height,
    );

    _outputs.push(fee_box);

    // create transaction
    let tx = chain::Transaction::new(_inputs, vec![], _outputs);

    Ok(JsValue::from_serde(&tx).unwrap())
}

#[wasm_bindgen]
pub fn parse_hd_path(path: &str) -> Vec<u32> {
    let hd_path = StandardHDPath::try_from(path).unwrap();

    vec![
        hd_path.purpose().as_value().as_number(),
        hd_path.coin_type(),
        hd_path.account(),
        hd_path.change(),
        hd_path.index(),
    ]
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use sigma_tree::{chain, ErgoTree};
    use sigma_tree::serialization::serializable::*;
    use wasm_bindgen::JsValue;

    #[test]
    pub fn address_validation() {
        let result = super::Address::validate(&"we");
    }

    fn input_from_id(id: &str) -> chain::Input {
        let id_bytes = chain::Base16DecodedBytes::try_from(id.to_string()).unwrap();
        let digest = chain::Digest32::try_from(id_bytes).unwrap();
        let box_id: chain::BoxId = chain::BoxId(digest);

        chain::Input {
            box_id,
            spending_proof: chain::ProverResult {
                proof: chain::ProofBytes::Empty,
                extension: chain::ContextExtension::empty(),
            },
        }
    }

    fn create_output(value: u64, ergo_tree: &str, height: u32) -> chain::ErgoBoxCandidate {
        let ergo_tree_bytes = chain::Base16DecodedBytes::try_from(ergo_tree.to_string()).unwrap();
        let tree = ErgoTree::sigma_parse_bytes(ergo_tree_bytes.0).unwrap();

        chain::ErgoBoxCandidate::new(
            chain::box_value::BoxValue::new(value).unwrap(),
            tree,
            height,
        )
    }

    #[test]
    pub fn tx_id_test() {
        let inputs = vec![];
        let outputs = vec![];

        let tx = chain::Transaction::new(inputs, vec![], outputs);

        let js = serde_json::to_string(&tx);
        println!("{:?}", js);
    }
}
