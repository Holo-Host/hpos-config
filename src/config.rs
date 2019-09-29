use ed25519_dalek::*;
//use rand::{Rng, RngCore, CryptoRng};

use rand;
use serde_repr::*;
use serde::ser::{Serialize, Serializer};

use crate::error::*;

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
    ) -> Result<Self, ConfigurationError> {
        let seed = match seed_maybe {
            Some(s) => s,
            None => {
                let out: [u8; HOLO_ENTROPY_SIZE] = rand::random();
                out
            }
        };

        let admin_keypair = admin_key_from(&email, &password)?;

        let admin = Admin {
            email: EmailAddress(email),
            public_key: hcid::HcidEncoding::with_kind("hca0")?
                .encode(&admin_keypair.public.to_bytes())?,
        };

        Ok(Config {
            version: Version::V1,
            admins: vec![admin],
            seed: hcid::HcidEncoding::with_kind("hcc0")?.encode(&seed)?,
        })
    }
}

/// admin_key_from -- Stretches the email (salt) + password, generates "admin" keypair
///
/// TODO: We must be able to generate a sequence of unique admin keypairs for each unique HoloPort.
/// This is also required for (later) when we support DPKI-generated entropy/keypairs.
pub fn admin_key_from(email: &str, password: &str) -> Result<Keypair, ConfigurationError> {
    // Extend the email address to a 512-bit salt using SHA-512. This prevents very short
    // email addresses (eg. a@b.ca) from triggering salt size related failures in argon2.
    let salt = Sha512::digest(email.as_bytes());

    // Extend the hashed email salt + password into a seed for the admin Keypair
    keypair_from_seed(&argon2::hash_raw(
        &password.as_bytes(),
        &salt,
        &ARGON2_CONFIG,
    )?)
}

pub fn keypair_from_seed(seed: &[u8]) -> Result<Keypair, ConfigurationError> {
    let secret: SecretKey = SecretKey::from_bytes(seed)?;
    let public: PublicKey = (&secret).into();
    Ok(Keypair { public, secret })
}
