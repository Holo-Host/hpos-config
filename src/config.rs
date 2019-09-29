use ed25519_dalek::*;
use failure::Error;

use serde::ser::{Serialize, Serializer};
use serde_derive::*;
use serde_repr::*;

pub const ARGON2_CONFIG: argon2::Config = argon2::Config {
    variant: argon2::Variant::Argon2id,
    version: argon2::Version::Version13,
    mem_cost: 1 << 16, // 64 MB
    time_cost: 2,
    lanes: 4,
    thread_mode: argon2::ThreadMode::Parallel,
    secret: &[],
    ad: "holo-config-v1".as_bytes(),
    hash_length: 32,
};

pub const HOLO_ENTROPY_SIZE: usize = 32;

#[derive(Debug, Serialize_repr)]
#[repr(u8)]
pub enum Version {
    V1 = 1,
}

#[derive(Debug, Serialize)]
struct EmailAddress(String);

#[derive(Debug, Serialize)]
pub struct Admin {
    email: EmailAddress,
    public_key: String,
}

#[derive(Debug, Serialize)]
pub struct Config {
    version: Version,
    admins: Vec<Admin>,
    seed: String,
}

impl Config {
    /// new -- Create a new config from provided email/password, + optional seed entropy
    ///
    /// Deduces and creates the admin keys, and creates the config.
    pub fn new(
        email: String,
        password: String,
        seed_maybe: Option<[u8; HOLO_ENTROPY_SIZE]>,
    ) -> Result<Self, Error> {
        let seed = match seed_maybe {
            Some(s) => s,
            None => {
                let out: [u8; HOLO_ENTROPY_SIZE] = rand::random();
                out
            }
        };

        let admin_public_key = public_key_from(&email, &password)?;

        let admin = Admin {
            email: EmailAddress(email),
            public_key: hcid::HcidEncoding::with_kind("hca0")?
                .encode(&admin_public_key.to_bytes())?,
        };

        Ok(Config {
            version: Version::V1,
            admins: vec![admin],
            seed: hcid::HcidEncoding::with_kind("hcc0")?.encode(&seed)?,
        })
    }
}

pub fn public_key_from(email: &str, password: &str) -> Result<PublicKey, Error> {
    // Extend the email address to a 512-bit salt using SHA-512. This prevents
    // very short email addresses (such as a@b.ca) from triggering salt size
    // related failures in Argon2.
    let salt = Sha512::digest(email.as_bytes());

    // TODO: Argon2 secret (see ARGON2_CONFIG) should be set to Holochain public key
    let hash = &argon2::hash_raw(
        &password.as_bytes(),
        &salt,
        &ARGON2_CONFIG,
    )?;

    Ok(PublicKey::from(&SecretKey::from_bytes(hash)?))
}
