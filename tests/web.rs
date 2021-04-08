//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;


use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;
use web_sys::console;

use ergowallet_wasm::*;
use ergo_lib::chain::ergo_box::BoxValue;
use ergo_lib::chain::contract::Contract;
use ergo_lib::ergotree_ir::serialization::SigmaSerializable;
use ergo_lib::ergotree_ir::address::{AddressEncoder, NetworkPrefix};
use ergo_lib::ergotree_interpreter::sigma_protocol::private_input::DlogProverInput;


//wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
pub fn random_secret_generation() {
    let secret = DlogProverInput::random();
    let js_value = JsValue::from_serde(secret.w.to_bytes().as_slice()).unwrap();
    console::log_1(&"secret is ".into());
    console::log_1(&js_value);
}


#[wasm_bindgen_test]
pub fn ergo_tree_p2pk_serialization() {
    let encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
    let address = encoder
        .parse_address_from_str("9grLsBktkgPuNWAHH4QTb5GPxL4eH5mFgwqcLaUaMWkU9R7ZqKu")
        .unwrap();

    let contract = Contract::pay_to_address(&address).unwrap();
    let tree_bytes = contract.ergo_tree().sigma_serialize_bytes();

    let correct_tree = vec![
        0, 8, 205, 3, 51, 44, 148, 61, 231, 78, 149, 5, 42, 24, 133, 168, 248, 27, 78, 229, 222,
        113, 89, 161, 129, 66, 225, 90, 46, 207, 24, 205, 23, 3, 104, 219,
    ];
    assert_eq!(correct_tree, tree_bytes);
    // console::log_1(&js_value);
}
