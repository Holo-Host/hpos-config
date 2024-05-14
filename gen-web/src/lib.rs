use ed25519_dalek::PublicKey;
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
    revocation_pub_key: String,
    holoport_id: Vec<u8>,
    derivation_path: String,
    device_bundle: String,
    device_pub_key: String,
) -> Result<JsValue, Error> {
    let device_pub_key: PublicKey = base64::decode_config(&device_pub_key, base64::URL_SAFE_NO_PAD)
        .map(|bytes| PublicKey::from_bytes(&bytes))??;
    let holoport_id = PublicKey::from_bytes(&holoport_id)?;
    let (config, public_key) = Config::new(
        email,
        password,
        registration_code,
        revocation_pub_key,
        holoport_id,
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
    revocation_pub_key: String,
    holoport_id: Vec<u8>,
    derivation_path: String,
    device_bundle: String,
    device_pub_key: String,
) -> Result<JsValue, JsValue> {
    match config_raw(
        email,
        password,
        registration_code,
        revocation_pub_key,
        holoport_id,
        derivation_path,
        device_bundle,
        device_pub_key,
    ) {
        Ok(js_val) => Ok(js_val),
        Err(e) => Err(e.to_string().into()),
    }
}
