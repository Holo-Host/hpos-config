use arrayref::array_ref;
use failure::Error;
use hpos_config_core::{
    config::{Seed, SEED_SIZE},
    public_key, Config,
};
use serde::*;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
pub struct ConfigData {
    config: String,
    url: String,
}

// https://github.com/rustwasm/wasm-bindgen/issues/1004
fn config_raw(
    email: String,
    password: String,
    registration_code: String,
    derivation_path: String,
    seed: String,
) -> Result<JsValue, Error> {
    // deserialize seed
    let seed: Seed = base64::decode(&seed).map(|bytes| array_ref!(bytes, 0, SEED_SIZE).clone())?;
    let (config, public_key) = Config::new_v2(
        email,
        password,
        registration_code,
        derivation_path,
        Some(seed),
    )?;

    let config_data = ConfigData {
        config: serde_json::to_string_pretty(&config)?,
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
    seed: String,
) -> Result<JsValue, JsValue> {
    match config_raw(email, password, registration_code, derivation_path, seed) {
        Ok(js_val) => Ok(js_val),
        Err(e) => Err(e.to_string().into()),
    }
}
