extern crate holo_config;
extern crate wasm_bindgen;
extern crate wee_alloc;
extern crate serde_json;
extern crate failure;

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use wasm_bindgen::prelude::*;

pub type JsResult<T> = Result<T, JsValue>;
macro_rules! jserr {
    ($code:expr) => {
        match $code {
            Ok(v) => Ok(v),
            Err(e) => Err(JsValue::from_str(&format!("{:?}", e))),
        }
    };
}

#[wasm_bindgen]
pub struct Config(holo_config::Config, holo_config::SigningPublicKey);

#[wasm_bindgen]
pub struct Seed(holo_config::Seed);

#[wasm_bindgen]
impl Config {
    #[wasm_bindgen(constructor)]
    pub fn new(
        email: String,
        password: String,
        maybe_seed: Option<Seed>,
        encrypt: bool
    ) -> JsResult<Config> {

        let config_result = jserr!(
            holo_config::Config::new(
                email, password, maybe_seed.map(|s| s.0), encrypt
            )
        )?;
        Ok(
            Config(
                config_result.0,
                config_result.1,
            )
        )
    }
}
