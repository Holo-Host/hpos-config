
extern crate argon2;
extern crate ed25519_dalek;
extern crate rand;
extern crate hcid;
extern crate base64;
extern crate crypto;

// Support De/Serializing from/to JSON
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

//#[macro_use]
//extern crate arrayref;

pub use error::*;
pub use configuration::*;

mod error;
mod configuration;
