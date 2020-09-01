#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
extern crate wasm_bindgen;

use wasm_bindgen_test::*;
use ergowallet_wasm::*;

const MESSAGE: &'static str = "Secret Message to Encrypt";
const PASSWORD: &'static str = "Ergo Wallet Password!";
const SALT: [u8; 32] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1];
const NONCE: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1];

#[wasm_bindgen_test]
fn encrypt_decrypt_with_password_success() {
    let encrypted = password_encrypt(PASSWORD, &SALT, &NONCE, MESSAGE.as_bytes()).unwrap();
    let encrypted: Vec<u8> = encrypted.into_serde().unwrap();
    let decrypted = password_decrypt(PASSWORD, &encrypted).unwrap();
    let decrypted: Vec<u8> = decrypted.into_serde().unwrap();
    assert_eq!(MESSAGE.as_bytes(), decrypted.as_slice());
}

#[wasm_bindgen_test]
fn encrypt_decrypt_invalid_password() {
    const INVALID_PASSWORD: &'static str = "Invalid Password";
    let encrypted = password_encrypt(PASSWORD, &SALT, &NONCE, MESSAGE.as_bytes()).unwrap();
    let encrypted: Vec<u8> = encrypted.into_serde().unwrap();
    assert!(password_decrypt(INVALID_PASSWORD, &encrypted).is_err());
}