use ed25519_dalek::VerifyingKey;
use failure::Error;
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
    revocation_pub_key: Vec<u8>,
    derivation_path: String,
    device_bundle: String,
    device_pub_key: Vec<u8>,
) -> Result<JsValue, Error> {
    let device_pub_key: VerifyingKey = VerifyingKey::from_bytes(
        &device_pub_key
            .try_into()
            .expect("Expected a Vec of length 32"),
    )?;

    let revocation_pub_key = VerifyingKey::from_bytes(
        &revocation_pub_key
            .try_into()
            .expect("Expected a Vec of length 32"),
    )?;

    let (config, public_key) = Config::new(
        email,
        password,
        registration_code,
        revocation_pub_key,
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
    revocation_pub_key: Vec<u8>,
    derivation_path: String,
    device_bundle: String,
    device_pub_key: Vec<u8>,
) -> Result<JsValue, JsValue> {
    match config_raw(
        email,
        password,
        registration_code,
        revocation_pub_key,
        derivation_path,
        device_bundle,
        device_pub_key,
    ) {
        Ok(js_val) => Ok(js_val),
        Err(e) => Err(e.to_string().into()),
    }
}
