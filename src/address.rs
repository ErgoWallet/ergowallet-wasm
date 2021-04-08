use wasm_bindgen::prelude::*;

// use ergo_lib::chain::address::{AddressEncoder, NetworkPrefix};
// use ergo_lib::sigma_protocol::sigma_boolean::ProveDlog;
// use ergo_lib::serialization::SigmaSerializable;
use ergo_lib::chain;
use ergo_lib::ergotree_ir::sigma_protocol::sigma_boolean::ProveDlog;
use ergo_lib::ergotree_ir::address::{NetworkPrefix, AddressEncoder};
use ergo_lib::ergotree_ir::serialization::SigmaSerializable;
use ergo_lib::ergotree_ir::address::AddressTypePrefix::P2PK;

#[wasm_bindgen]
pub struct Address {
    address: String,
}

#[wasm_bindgen]
impl Address {
    pub fn get_addr(&self) -> String {
        return self.address.clone();
    }

    pub fn validate(address: &str) -> bool {
        let encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
        let result = encoder.parse_address_from_str(address);
        match result {
            Ok(_addr) => true,
            Err(_err) => false,
        }
    }

    pub fn from_public_key(pub_key: &[u8]) -> Address {
        let mut content_bytes: Vec<u8> = vec![];
        content_bytes.extend_from_slice(pub_key);

        let p2pk_address = ergo_lib::ergotree_ir::address::Address::P2PK(ProveDlog::sigma_parse_bytes(content_bytes).unwrap());
        let encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
        Address {
            address: encoder.address_to_str(&p2pk_address),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use wasm_bindgen::JsValue;

    #[test]
    pub fn address_validation() {
        let result = super::Address::validate(&"we");
        assert!(result);
    }
}