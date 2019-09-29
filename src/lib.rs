extern crate argon2;
extern crate base64;
extern crate crypto;
extern crate ed25519_dalek;
extern crate hcid;
extern crate rand;

// Support De/Serializing from/to JSON
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

//#[macro_use]
//extern crate arrayref;

pub use configuration::*;
pub use error::*;

mod configuration;
mod error;
