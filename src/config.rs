use holochain_dpki::SEED_SIZE;

use ed25519_dalek::*;
use failure::Error;
use rand::{rngs::OsRng, Rng};
use serde::*;

fn to_base64<T, S>(x: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: AsRef<[u8]>,
    S: Serializer,
{
    serializer.serialize_str(&base64::encode_config(x.as_ref(), base64::STANDARD_NO_PAD))
}

const ARGON2_CONFIG: argon2::Config = argon2::Config {
    variant: argon2::Variant::Argon2id,
    version: argon2::Version::Version13,
    mem_cost: 1 << 16, // 64 MB
    time_cost: 2,
    lanes: 4,
    thread_mode: argon2::ThreadMode::Parallel,
    secret: &[],
    ad: b"holo-config v1",
    hash_length: SECRET_KEY_LENGTH as u32,
};

pub type Seed = [u8; SEED_SIZE];

#[derive(Debug, Serialize)]
pub struct Admin {
    email: String,
    #[serde(serialize_with = "to_base64")]
    public_key: PublicKey,
}

#[derive(Debug, Serialize)]
pub enum Config {
    #[serde(rename = "v1")]
    V1 {
        #[serde(serialize_with = "to_base64")]
        seed: Seed,
        admins: Vec<Admin>,
    },
}

impl Config {
    pub fn new(email: String, password: String, maybe_seed: Option<Seed>) -> Result<Self, Error> {
        let seed = match maybe_seed {
            Some(s) => s,
            None => OsRng::new()?.gen::<Seed>(),
        };

        let admin = Admin {
            email: email.clone(),
            public_key: public_key_from(&email, &password)?,
        };

        Ok(Config::V1 {
            admins: vec![admin],
            seed: seed,
        })
    }
}

fn public_key_from(salt: &str, password: &str) -> Result<PublicKey, Error> {
    // This allows to use salt shorter than 8 bytes.
    let salt_hash = Sha512::digest(salt.as_bytes());

    let mut config = ARGON2_CONFIG.clone();
    // TODO: should be set to Holochain public key.
    config.secret = &[];

    let hash = &argon2::hash_raw(&password.as_bytes(), &salt_hash, &config)?;

    Ok(PublicKey::from(&SecretKey::from_bytes(hash)?))
}
