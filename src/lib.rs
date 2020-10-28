#[macro_use]
extern crate serde_derive;

use std::convert::{TryFrom, TryInto};

use hdpath::StandardHDPath;
use wasm_bindgen::prelude::*;
use web_sys::console;

pub use address::*;
pub use key_manager::*;
pub use password_crypto::*;
pub use transaction::*;


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
const MINERS_FEE_MAINNET_ADDRESS: &str =
    "2iHkR7CWvD1R4j1yZg5bkeDRQavjAaVPeTDFGGLZduHyfWMuYpmhHocX8GJoaieTx78FntzJbCBVL6rf96ocJoZdmWBL2fci7NqWgAirppPQmZ7fN9V6z13Ay6brPriBKYqLp1bT2Fk4FkFLCfdPpe";


#[wasm_bindgen(js_name = "parseHdPath")]
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

    use ergo_lib::chain::transaction::Transaction;
    use ergo_lib::chain::ergo_box::ErgoBoxCandidate;
    use ergo_lib::chain::ergo_box::box_value::BoxValue;
    use ergo_lib::chain::ergo_box::register::NonMandatoryRegisters;
    use ergo_lib::ErgoTree;
    use ergo_lib::serialization::SigmaSerializable;
    use ergo_lib::chain::Base16DecodedBytes;

    fn create_output(value: u64, ergo_tree: &str, height: u32) -> ErgoBoxCandidate {
        let ergo_tree_bytes = Base16DecodedBytes::try_from(ergo_tree.to_string()).unwrap();
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
