extern crate wasm_bindgen;

use ed25519_dalek::Signature;
use holo_config_core::AdminPublicKey;
use std::str::FromStr;
use wasm_bindgen::prelude::*;

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub type JsResult<T> = Result<T, JsValue>;

macro_rules! jserr {
    ($code:expr) => {
        match $code {
            Ok(v) => Ok(v),
            Err(e) => Err(JsValue::from_str(&format!("{:?}", e))),
        }
    };
}

/// AdminVerifier stores an AdminPublicKey and supports message signature verification
#[wasm_bindgen]
pub struct AdminVerifier(AdminPublicKey);

#[wasm_bindgen]
impl AdminVerifier {
    #[wasm_bindgen(constructor)]
    pub fn new(
        hcapk: &str		// HcA... Admin Public Key
    ) -> Result<AdminVerifier, JsValue> {
        let public_key = jserr!(AdminPublicKey::from_str(hcapk))?;
        Ok(Self(public_key))
    }

    #[wasm_bindgen]
    pub fn verify(
        &self,
        message: &[u8],		// bytes
        signature: &str		// Base-64, unpadded
    ) -> Result<bool, JsValue> {
        let signature_bytes = jserr!(base64::decode_config( &signature, base64::STANDARD_NO_PAD ))?;
        let signature = jserr!(Signature::from_bytes( &signature_bytes ))?;
        Ok(self.0.verify(message, &signature))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use hcid::HcidEncoding;
    use holo_config_core::admin_keypair_from;
    use ed25519_dalek::PublicKey;

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

        // Get the AdminPublicKey and Signature for a message, and test bindgen verify API
        let pubkey = AdminPublicKey(keypair.public);
        let pubkey_str = format!("{}", pubkey);

        let sig_str = base64::encode_config(&sig.to_bytes().as_ref(), base64::STANDARD_NO_PAD);

        println!("Admin PublicKey (hcid): {:?}", &pubkey_str);
        println!("Message:                {:?}", &message);
        println!("Signature (base-64):    {:?}", &sig_str);

        // Ensure that underlying AdminPublicKey.verify works (we test the wasm_bindgen via JS)
        assert_eq!(pubkey.verify( &message.as_bytes(), &sig ), true );
        assert_eq!(pubkey.verify( &message.as_bytes()[1..], &sig ), false );
    }
}
