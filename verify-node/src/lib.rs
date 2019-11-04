extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

use ed25519_dalek::{
    PublicKey, Signature,
};

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




// Given an Admin public key, verify a signature
#[wasm_bindgen]
pub fn verify(
    pubkey: &str,		// Base-64
    message: &[u8],		// bytes
    signature: &str,		// Base-64
) -> JsResult<bool> {
    let pubkey_bytes = jserr!(base64::decode_config( &pubkey, base64::STANDARD_NO_PAD ))?;
    let pubkey: PublicKey = jserr!(PublicKey::from_bytes( &pubkey_bytes ))?;
    let signature_bytes = jserr!(base64::decode_config( &signature, base64::STANDARD_NO_PAD ))?;
    let signature = jserr!(Signature::from_bytes( &signature_bytes ))?;

    Ok(verify_pubkey_signature( &pubkey, message, &signature ))
}

pub fn verify_pubkey_signature(
    pubkey: &PublicKey,
    message: &[u8],
    signature: &Signature,
) -> bool {
    match pubkey.verify( message, signature ) {
        Ok(()) => true,
        other => {
            println!("Invalid signature: {:?}", other);
            false
        },
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use hcid::HcidEncoding;
    use holo_config_core::{
        admin_keypair_from,
    };

    #[test]
    fn verify_smoke() {
        let agent_id = "HcSCjwu4wIi4BawccpoEINNfsybv76wrqoJe39y4KNAO83gsd87mKIU7Tfjy7ci";
        let agent_pubkey = PublicKey::from_bytes(
            &HcidEncoding::with_kind("hcs0").unwrap()
                .decode( &agent_id ).unwrap()
        ).unwrap();

        // Get an Admin signing key, and sign something
        let keypair = admin_keypair_from(
            agent_pubkey,
            "a@b.ca".as_ref(),
            "password".as_ref()
        ).unwrap();
        let message = "Of the increase of his government and peace there shall be no end";
        let sig = keypair.sign( message.as_bytes() );

        // Get the PublicKey and Signatures as Base-64, and test bindgen verify API
        let pubkey_string = base64::encode_config(
            &keypair.public.to_bytes().as_ref(), base64::STANDARD_NO_PAD);
        let sig_string = base64::encode_config(
            &sig.to_bytes().as_ref(), base64::STANDARD_NO_PAD);

        println!("Admin PublicKey (base-64): {:?}", pubkey_string);
        println!("Message: {:?}", message);
        println!("Signature: {:?}", sig_string);

        // Ensure that verify detects correct and incorrect messages and keys
        assert_eq!(verify( pubkey_string.as_ref(),
                           &message.as_bytes(), sig_string.as_ref()),
                   Ok(true));
        assert_eq!(verify( pubkey_string.as_ref(),
                           &message.as_bytes()[1..], sig_string.as_ref()),
                   Ok(false));

    }
}
