use arrayref::array_ref;
use ed25519_dalek::*;
use failure::Error;
use rand::{rngs::OsRng, Rng};
use serde::*;

const SEED_SIZE: usize = 32;

fn public_key_from_base64<'de, D>(deserializer: D) -> Result<PublicKey, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)
        .and_then(|s| {
            base64::decode_config(&s, base64::STANDARD_NO_PAD)
                .map_err(|err| de::Error::custom(err.to_string()))
        })
        .map(|bytes| PublicKey::from_bytes(&bytes))
        .and_then(|maybe_key| maybe_key.map_err(|err| de::Error::custom(err.to_string())))
}

fn seed_from_base64<'de, D>(deserializer: D) -> Result<Seed, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)
        .and_then(|s| base64::decode(&s).map_err(|err| de::Error::custom(err.to_string())))
        .map(|bytes| array_ref!(bytes, 0, SEED_SIZE).clone())
}

fn to_base64<T, S>(x: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: AsRef<[u8]>,
    S: Serializer,
{
    serializer.serialize_str(&base64::encode_config(x.as_ref(), base64::STANDARD_NO_PAD))
}

const ARGON2_ADDITIONAL_DATA: &[u8] = b"hpos-state admin ed25519 key v1";

pub type Seed = [u8; SEED_SIZE];

#[derive(Debug, Deserialize, Serialize)]
pub struct Admin {
    email: String,
    #[serde(
        deserialize_with = "public_key_from_base64",
        serialize_with = "to_base64"
    )]
    public_key: PublicKey,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    admin: Admin,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum State {
    #[serde(rename = "v1")]
    V1 {
        #[serde(deserialize_with = "seed_from_base64", serialize_with = "to_base64")]
        seed: Seed,
        config: Config,
    },
}

impl State {
    pub fn new(
        email: String,
        password: String,
        maybe_seed: Option<Seed>,
    ) -> Result<(Self, PublicKey), Error> {
        let seed = match maybe_seed {
            None => OsRng::new()?.gen::<Seed>(),
            Some(s) => s,
        };

        let holochain_secret_key = SecretKey::from_bytes(&seed)?;
        let holochain_public_key = PublicKey::from(&holochain_secret_key);

        let admin_keypair = admin_keypair_from(holochain_public_key, &email, &password)?;
        let admin = Admin {
            email: email.clone(),
            public_key: admin_keypair.public,
        };

        Ok((
            State::V1 {
                seed,
                config: Config { admin },
            },
            holochain_public_key,
        ))
    }

    pub fn admin_public_key(&self) -> PublicKey {
        match self {
            State::V1{seed: _, config: c} => c.admin.public_key,
            _ => unreachable!()
        }
    }
}

pub fn admin_keypair_from(
    holochain_public_key: PublicKey,
    email: &str,
    password: &str,
) -> Result<Keypair, Error> {
    // This allows to use email addresses shorter than 8 bytes.
    let salt = Sha512::digest(email.as_bytes());
    let mut hash = [0; SEED_SIZE];

    argon2min::Argon2::new(2, 4, 1 << 16, argon2min::Variant::Argon2id)?.hash(
        &mut hash,
        password.as_bytes(),
        &salt,
        &holochain_public_key.to_bytes(),
        ARGON2_ADDITIONAL_DATA,
    );

    let secret_key = SecretKey::from_bytes(&hash)?;
    let public_key = PublicKey::from(&secret_key);

    Ok(Keypair{
        public: public_key,
        secret: secret_key,
    })
}
