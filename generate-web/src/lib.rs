use failure::Error;
use holo_config_core::{public_key, Config};
use serde::*;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
pub struct ConfigData {
    config: String,
    url: String
}

// https://github.com/rustwasm/wasm-bindgen/issues/1004
fn config_raw(email: String, password: String, device_name: Option<String>) -> Result<JsValue, Error> {
    let (config, public_key) = Config::new(email, password, device_name, None)?;

    let config_data = ConfigData {
        config: serde_json::to_string_pretty(&config)?,
        url: public_key::to_url(&public_key)?.into_string()
    };

    Ok(JsValue::from_serde(&config_data)?)
}

#[wasm_bindgen]
pub fn config(email: String, password: String, device_name: Option<String>) -> Result<JsValue, JsValue> {
    match config_raw(email, password, device_name) {
        Ok(js_val) => Ok(js_val),
        Err(e) => Err(e.to_string().into())
    }
}