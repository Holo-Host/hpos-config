use ed25519_dalek::VerifyingKey;
use failure::{format_err, Error};
use hpos_config_core::{public_key, Config};
use serde::*;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
pub struct ConfigData {
    config: String,
    id: String,
    url: String,
}

// https://github.com/rustwasm/wasm-bindgen/issues/1004
fn config_raw(
    email: String,
    password: String,
    registration_code: String,
    derivation_path: String,
    device_bundle: String,
    device_pub_key: String,
) -> Result<JsValue, Error> {
    let bytes: [u8; 32] =
        match (base64::decode_config(device_pub_key, base64::URL_SAFE_NO_PAD)?)[0..32].try_into() {
            Ok(b) => b,
            Err(_) => return Err(format_err!("Device pub key is not 32 bytes in size")),
        };

    let device_pub_key: VerifyingKey = VerifyingKey::from_bytes(&bytes)?;

    let (config, public_key) = Config::new_v2(
        email,
        password,
        registration_code,
        derivation_path,
        device_bundle,
        device_pub_key,
    )?;

    let config_data = ConfigData {
        config: serde_json::to_string_pretty(&config)?,
        id: public_key::to_base36_id(&public_key),
        url: public_key::to_url(&public_key)?.to_string(),
    };

    Ok(JsValue::from_serde(&config_data)?)
}

#[wasm_bindgen]
pub fn config(
    email: String,
    password: String,
    registration_code: String,
    derivation_path: String,
    device_bundle: String,
    device_pub_key: String,
) -> Result<JsValue, JsValue> {
    match config_raw(
        email,
        password,
        registration_code,
        derivation_path,
        device_bundle,
        device_pub_key,
    ) {
        Ok(js_val) => Ok(js_val),
        Err(e) => Err(e.to_string().into()),
    }
}
