//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;

use wasm_bindgen_test::*;
use wasm_bindgen::JsValue;
use web_sys::console;
use ergowallet_wasm::{Transaction, TxOutput, TxInput};
use sigma_tree::chain::ErgoBoxCandidate;
use sigma_ser::serializer::SigmaSerializable;

// wasm_bindgen_test_configure!(run_in_browser);


#[wasm_bindgen_test]
pub fn tx_creation() {
    let inputs: Vec<TxInput> = vec![
        TxInput { box_id: "626925e6a7bb08e3b7cf73de2e71a98966e881e7fc0c54fbbc94b83c79de8c19".to_string() },
        TxInput { box_id: "626925e6a7bb08e3b7cf73de2e71a98966e881e7fc0c54fbbc94b83c79de8c19".to_string() }
    ];

    let outputs: Vec<TxOutput> = vec![
        TxOutput { value: "334412".to_string(), address: "9hzP24a2q8KLPVCUk7gdMDXYc7vinmGuxmLp5KU7k9UwptgYBYV".to_string()}
    ];

    let js_value = inputs
        .into_iter()
        .map(|x| JsValue::from_serde(&x).unwrap())
        .collect::<Vec<JsValue>>()
        .into_boxed_slice();

    let js_outputs = outputs
        .into_iter()
        .map(|x| JsValue::from_serde(&x).unwrap())
        .collect::<Vec<JsValue>>()
        .into_boxed_slice();

     let result = Transaction::create(js_value, js_outputs, 100u64, 0);

    // let s = format!("{:?}", result);

    console::log_1(&result);

    let bytes: Vec<u8> = Transaction::serialize(&result);

}

#[wasm_bindgen_test]
pub fn ergo_tree_p2pk_serialization() {
    let encoder = sigma_tree::chain::AddressEncoder::new(sigma_tree::chain::NetworkPrefix::Mainnet);
    let address = encoder.parse_address_from_str("9grLsBktkgPuNWAHH4QTb5GPxL4eH5mFgwqcLaUaMWkU9R7ZqKu").unwrap();

    let contract = sigma_tree::chain::Contract::pay_to_address(address).unwrap();
    let tree_bytes = contract.get_ergo_tree().sigma_serialise_bytes();

    let correct_tree = vec![
        0,   8, 205,   3,  51,  44, 148,  61,
        231, 78, 149,   5,  42,  24, 133, 168, 248,
        27, 78, 229, 222, 113,  89, 161, 129,  66,
        225, 90,  46, 207,  24, 205,  23,   3, 104,
        219
    ];
    assert_eq!(correct_tree, tree_bytes);
    // console::log_1(&js_value);
}
