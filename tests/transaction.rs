#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
extern crate wasm_bindgen;

use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;
use web_sys::console;

use ergowallet_wasm::*;

use sigma_tree::sigma_protocol::DlogProverInput;
use sigma_tree::chain::{Base16EncodedBytes, Base16DecodedBytes};
use sigma_tree::chain::transaction::{Transaction, TxId};

use sigma_tree::chain::ergo_box::{ErgoBox, ErgoBoxCandidate};
use sigma_tree::wallet::secret_key::SecretKey;
use sigma_tree::chain::contract::Contract;
use sigma_tree::chain::ergo_box::box_value::BoxValue;
use sigma_tree::chain::ergo_box::register::NonMandatoryRegisters;
use sigma_tree::chain::input::{UnsignedInput, Input};
use sigma_tree::chain::prover_result::{ProverResult, ProofBytes};
use sigma_tree::chain::context_extension::ContextExtension;

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
                ergo_tree: Contract::pay_to_address(&address).unwrap().get_ergo_tree(),
                tokens: vec![],
                additional_registers: NonMandatoryRegisters::empty(),
                creation_height: 0,
            },
            TxId::zero(), 0)
    ];

    let outputs = vec![
        ErgoBoxCandidate {
            value: BoxValue::SAFE_USER_MIN,
            ergo_tree: Contract::pay_to_address(&address).unwrap().get_ergo_tree(),
            tokens: vec![],
            additional_registers: NonMandatoryRegisters::empty(),
            creation_height: 0,
        }
    ];
    // Spend available box
    let inputs = boxes
        .iter()
        .map(|b| Input {
            box_id: b.box_id().clone(),
            spending_proof: ProverResult {
                proof: ProofBytes::Empty,
                extension: ContextExtension::empty(),
            },
        })
        .collect();

    let tx = Transaction::new(
        inputs,
        vec![],
        outputs,
    );

    let js_boxes = boxes
        .into_iter()
        .map(|x| JsValue::from_serde(&x).unwrap())
        .collect::<Vec<JsValue>>()
        .into_boxed_slice();

    sign_tx(
        js_secrets,
        js_boxes,
        &JsValue::from_serde(&tx).unwrap(),
    );
}