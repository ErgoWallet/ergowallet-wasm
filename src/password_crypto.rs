use wasm_bindgen::prelude::*;

use hmac::Hmac;
use pbkdf2::pbkdf2;
use sha2::{Sha256, Sha512};
use aes_gcm::Aes256Gcm;
use aes_gcm::aead::{Aead, NewAead, generic_array::GenericArray, AeadMut};

const SALT_SIZE: usize = 32;
const NONCE_SIZE: usize = 12;
const KEY_SIZE: usize = 32;
const ROUNDS: u32 = 10_000;

#[wasm_bindgen]
pub fn password_encrypt(
    password: &str,
    salt: &[u8],
    nonce: &[u8],
    data: &[u8],
) -> Result<JsValue, JsValue> {
    if salt.len() != SALT_SIZE {
        return Err(JsValue::from(&format!("Invalid Salt Size, expected {} bytes", SALT_SIZE)));
    }

    // Derive key
    let mut key = vec![0u8; KEY_SIZE];
    pbkdf2::<Hmac<Sha512>>(password.as_bytes(), salt, ROUNDS, &mut key);

    // Encrypt by AES
    let cipher = Aes256Gcm::new(GenericArray::from_slice(&key[..]));
    let result = cipher.encrypt(
        GenericArray::from_slice(&nonce[..]),
        data,
    );

    match result {
        Ok(encrypted) => {
            let mut output = vec![];
            output.extend_from_slice(&salt);
            output.extend_from_slice(&nonce);
            output.extend_from_slice(&encrypted);

            JsValue::from_serde(&output).map_err(|e| JsValue::from_str(&format!("{:?}", e)))
        },
        Err(err) => Err(JsValue::from_str(&format!("Cannot encrypt the data: {:?}", err)))
    }
}

#[wasm_bindgen]
pub fn password_decrypt(password: &str, encrypted_data: &[u8]) -> Result<JsValue, JsValue> {

    // Extract meta information
    let salt = &encrypted_data[0..SALT_SIZE];
    let nonce = &encrypted_data[SALT_SIZE..(SALT_SIZE + NONCE_SIZE)];
    let encrypted = &encrypted_data[(SALT_SIZE + NONCE_SIZE)..];

    // Derive key
    let mut key = vec![0u8; KEY_SIZE];
    pbkdf2::<Hmac<Sha512>>(password.as_bytes(), salt, ROUNDS, &mut key);

    // Decrypt by AES
    let cipher = Aes256Gcm::new(GenericArray::from_slice(&key[..]));
    let result = cipher.decrypt(
        GenericArray::from_slice(&nonce[..]),
        encrypted
    );

    match result {
        Ok(decrypted) =>
            JsValue::from_serde(&decrypted).map_err(|e| JsValue::from_str(&format!("{:?}", e))),
        Err(_) =>
            Err(JsValue::from_str("Cannot decrypt the data"))
    }
}
