#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
extern crate wasm_bindgen;

use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;
use web_sys::console;

use ergowallet_wasm::*;
use ergo_lib::sigma_protocol::DlogProverInput;
use ergo_lib::wallet::secret_key::SecretKey;
use ergo_lib::chain::Base16EncodedBytes;
use ergo_lib::chain::ergo_box::{ErgoBox, ErgoBoxCandidate};
use ergo_lib::chain::ergo_box::box_value::BoxValue;
use ergo_lib::chain::contract::Contract;
use ergo_lib::chain::ergo_box::register::NonMandatoryRegisters;
use ergo_lib::chain::transaction::TxId;
use ergo_lib::chain::context_extension::ContextExtension;
use ergo_lib::chain::prover_result::{ProofBytes, ProverResult};
use ergo_lib::chain::input::{Input, UnsignedInput};


#[wasm_bindgen_test]
fn tx_sign_success() {
    // 1. Create random secret key
    let dpi = DlogProverInput::random();
    let secret1 = dpi.w.to_bytes();
    let secretKey = SecretKey::DlogSecretKey(dpi);
    let s: String = Base16EncodedBytes::new(secret1.as_slice()).into();

    console::log_1(&JsValue::from(&s));
    let js_secrets = [JsValue::from_serde(&s).unwrap()]
        .to_vec()
        .into_boxed_slice();
    let address = secretKey.get_address_from_public_image();

    // 2. Create available boxes
    let boxes = vec![
        ErgoBox::from_box_candidate(
            &ErgoBoxCandidate {
                value: BoxValue::new(BoxValue::SAFE_USER_MIN.as_u64() * 2).unwrap(),
                ergo_tree: Contract::pay_to_address(&address).unwrap().ergo_tree(),
                tokens: vec![],
                additional_registers: NonMandatoryRegisters::empty(),
                creation_height: 0,
            },
            TxId::zero(), 0)
    ];

    let outputs = vec![
        ErgoBoxCandidate {
            value: BoxValue::SAFE_USER_MIN,
            ergo_tree: Contract::pay_to_address(&address).unwrap().ergo_tree(),
            tokens: vec![],
            additional_registers: NonMandatoryRegisters::empty(),
            creation_height: 0,
        }
    ];
    // Spend available box
    let inputs = boxes
        .iter()
        .map(|b| UnsignedInput {
            box_id: b.box_id().clone(),
            extension: ContextExtension::empty(),
        })
        .collect();

    let tx = ergo_lib::chain::transaction::unsigned::UnsignedTransaction::new(
        inputs,
        vec![],
        outputs,
    );

    let js_boxes = boxes
        .into_iter()
        .map(|x| JsValue::from_serde(&x).unwrap())
        .collect::<Vec<JsValue>>()
        .into_boxed_slice();

    Transaction::sign(
        js_secrets,
        js_boxes,
        &JsValue::from_serde(&tx).unwrap(),
    );
}

#[wasm_bindgen_test]
pub fn tx_creation() {
    let inputs: Vec<TxInput> = vec![
        TxInput {
            box_id: "626925e6a7bb08e3b7cf73de2e71a98966e881e7fc0c54fbbc94b83c79de8c19".to_string(),
        },
        TxInput {
            box_id: "626925e6a7bb08e3b7cf73de2e71a98966e881e7fc0c54fbbc94b83c79de8c19".to_string(),
        },
    ];

    let outputs: Vec<TxOutput> = vec![TxOutput {
        assets: vec![AssetValue {
            token_id: "626925e6a7bb08e3b7cf73de2e71a98966e881e7fc0c54fbbc94b83c79de8c19".to_string(),
            amount: "1".to_string()
        }],
        value: BoxValue::SAFE_USER_MIN.as_u64().to_string(),
        address: "9hzP24a2q8KLPVCUk7gdMDXYc7vinmGuxmLp5KU7k9UwptgYBYV".to_string(),
    }];

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

    let result = Transaction::create(
        js_value, js_outputs, BoxValue::SAFE_USER_MIN.as_u64().clone(), 0).unwrap();

    console::log_1(&result.to_json().unwrap());
}
