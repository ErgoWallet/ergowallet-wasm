#[macro_use]
extern crate serde_derive;

use std::convert::{TryFrom, TryInto};

use hdpath::StandardHDPath;
use sigma_tree::serialization::*;
use sigma_tree::sigma_protocol;
use wasm_bindgen::prelude::*;
use web_sys::console;

pub use address::*;
pub use key_manager::*;
pub use password_crypto::*;
pub use transaction::*;

use sigma_tree::chain::prover_result::{ProverResult, ProofBytes};
use sigma_tree::chain::ergo_box::ErgoBoxCandidate;
use sigma_tree::chain::input::Input;
use sigma_tree::chain::ergo_box::box_id::BoxId;
use sigma_tree::chain::ergo_box::box_value::BoxValue;
use sigma_tree::chain::contract::Contract;
use sigma_tree::chain::address::{AddressEncoder, NetworkPrefix};
use sigma_tree::chain::transaction::Transaction;

mod key_manager;
mod address;
mod password_crypto;
mod utils;
mod transaction;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const MINER_ERGO_TREE: &str = "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304";


#[wasm_bindgen]
pub fn serialize_tx(js_tx: &JsValue) -> Vec<u8> {
    let tx: Transaction = js_tx.into_serde().unwrap();
    tx.sigma_serialise_bytes()
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
    use sigma_tree::chain::ergo_box::box_id::BoxId;
    use sigma_tree::chain::ergo_box::ErgoBoxCandidate;
    use sigma_tree::chain::prover_result::{ProverResult, ProofBytes};
    use sigma_tree::chain::input::Input;
    use sigma_tree::chain::transaction::Transaction;
    use sigma_tree::chain::ergo_box::box_value::BoxValue;
    use sigma_tree::chain::context_extension::ContextExtension;
    use sigma_tree::chain::ergo_box::register::NonMandatoryRegisters;


    fn create_output(value: u64, ergo_tree: &str, height: u32) -> ErgoBoxCandidate {
        let ergo_tree_bytes = chain::Base16DecodedBytes::try_from(ergo_tree.to_string()).unwrap();
        let tree = ErgoTree::sigma_parse_bytes(ergo_tree_bytes.0).unwrap();

        ErgoBoxCandidate {
            value: BoxValue::new(value).unwrap(),
            ergo_tree: tree,
            tokens: vec![],
            additional_registers: NonMandatoryRegisters::empty(),
            creation_height: height,
        }
    }

    #[test]
    pub fn tx_id_test() {
        let inputs = vec![];
        let outputs = vec![];

        let tx = Transaction::new(inputs, vec![], outputs);

        let js = serde_json::to_string(&tx);
        println!("{:?}", js);
    }
}
