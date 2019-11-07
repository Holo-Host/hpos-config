use failure::Error;
use hpos_state_core::{public_key, State};
use serde::*;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
pub struct StateData {
    state: String,
    url: String,
}

// https://github.com/rustwasm/wasm-bindgen/issues/1004
fn state_raw(email: String, password: String) -> Result<JsValue, Error> {
    let (state, public_key) = State::new(email, password, None)?;

    let state_data = StateData {
        state: serde_json::to_string_pretty(&state)?,
        url: public_key::to_url(&public_key)?.into_string(),
    };

    Ok(JsValue::from_serde(&state_data)?)
}

#[wasm_bindgen]
pub fn state(email: String, password: String) -> Result<JsValue, JsValue> {
    match state_raw(email, password) {
        Ok(js_val) => Ok(js_val),
        Err(e) => Err(e.to_string().into()),
    }
}
