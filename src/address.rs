use sigma_tree::{chain, ErgoTree};
use sigma_tree::serialization::*;
use sigma_tree::serialization::serializable::*;
use sigma_tree::sigma_protocol::sigma_boolean::ProveDlog;
use wasm_bindgen::prelude::*;


#[wasm_bindgen]
pub struct Address {
    address: String,
}

#[wasm_bindgen]
impl Address {
    pub fn get_addr(&self) -> String {
        return self.address.clone();
    }

    pub fn validate(address: &str) -> String {
        let encoder = chain::AddressEncoder::new(chain::NetworkPrefix::Mainnet);
        let result = encoder.parse_address_from_str(address);
        match result {
            Ok(addr) => String::new(),
            Err(err) => format!("{}", err),
        }
    }

    pub fn from_public_key(pub_key: &[u8]) -> Address {
        let mut content_bytes: Vec<u8> = vec![];
        content_bytes.extend_from_slice(pub_key);

        let p2pk_address: chain::Address =
            chain::Address::P2PK(ProveDlog::sigma_parse_bytes(content_bytes).unwrap());

        let encoder = chain::AddressEncoder::new(chain::NetworkPrefix::Mainnet);
        encoder.address_to_str(&p2pk_address);

        Address {
            address: encoder.address_to_str(&p2pk_address),
        }
    }
}