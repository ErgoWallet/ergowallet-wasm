use wasm_bindgen::prelude::*;
use web_sys::console;

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
