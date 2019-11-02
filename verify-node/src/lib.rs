extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

use ed25519_dalek::{
    PublicKey, Signature,
};
use hcid::HcidEncoding;

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

// this function changes a value by reference (borrowing and change)
#[wasm_bindgen]
pub fn alter(a: &mut [u8]) {
    a[1] = 12;
}

pub type JsResult<T> = Result<T, JsValue>;
macro_rules! jserr {
    ($code:expr) => {
        match $code {
            Ok(v) => Ok(v),
            Err(e) => Err(JsValue::from_str(&format!("{:?}", e))),
        }
    };
}

// Given a public key, verify a signature
#[wasm_bindgen]
pub fn verify(
    agent_id: String,		// HCID
    signature: String,		// Base-64
    message: &[u8],
) -> JsResult<bool> {

    let hcid_codec = jserr!(HcidEncoding::with_kind("hcs0"))?;
    let pubkey_bytes = jserr!(hcid_codec.decode( &agent_id ))?;
    let pubkey: PublicKey = jserr!(PublicKey::from_bytes( &pubkey_bytes ))?;
    let signature_bytes = jserr!(base64::decode( &signature ))?;
    let signature = jserr!(Signature::from_bytes( &signature_bytes ))?;

    pubkey.verify( message, &signature )
        .and(Ok(true))
        .or(Ok(false))
}
