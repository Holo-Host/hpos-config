// holo-config-js -- Javascript APIs for generating holo-config.json, signing Admin requests
// 
// See https://hackmd.io/RlGKKxltQk-97kkcwrA3-A for a summary of the APIs required.
// 
// The holo-config Config contains an entropy Seed, from which the Agent ID Signing keypair,
// and Admin API authentication keypair can be derived.
// 
// Config::new  - Generate a new Config, from either a random or supplied Seed
//   - Derives the Agent ID directly from the Seed
//   - Derives the Admin key, from the password, email (salt) and Agent ID (pepper)
//     - This keypair is unique to each HoloPort Agent ID, even if the same password/email is used
// 
// The HoloPort requires only the Admin Signing Public Key, in order to check the authenticity of
// incoming Admin requests.  The Admin UI (probably written in Javascript) requires the Admin
// Signing Private Key, in order to sign these requests.  Since the seed used to generate the Admin
// keypair is time-consuming to create (uses argon2), this derivation should only be done once, when
// the password/email and Agent ID become available.
// 

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
pub struct ConfigSeed(holo_config::ConfigSeed);

#[wasm_bindgen]
pub struct Config(holo_config::ConfigResult);
    
#[wasm_bindgen]
impl Config {
    #[wasm_bindgen(constructor)]
    pub fn new(
        email: String,
        password: String,
        maybe_seed: Option<ConfigSeed>,
        encrypt: bool
    ) -> JsResult<Config> {

        let config_result = jserr!(
            holo_config::Config::new(
                email, password, maybe_seed.map(|s| s.0), encrypt
            )
        )?;
        Ok(Config(config_result))
    }
}

