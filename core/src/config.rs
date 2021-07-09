use crate::public_key::holochain_pub_key_encoding;
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

fn public_key_from_dns_safe_base64<'de, D>(deserializer: D) -> Result<PublicKey, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)
        .and_then(|s| multibase::decode(&s[4..]).map_err(|err| de::Error::custom(err.to_string())))
        .map(|(_, bytes)| PublicKey::from_bytes(&bytes[..32]))
        .and_then(|maybe_key| maybe_key.map_err(|err| de::Error::custom(err.to_string())))
}

const ARGON2_ADDITIONAL_DATA: &[u8] = b"hpos-config admin ed25519 key v1";

fn public_key_to_dns_safe_base64<T, S>(x: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: AsRef<[u8]>,
    S: Serializer,
{
    serializer.serialize_str(&holochain_pub_key_encoding(x.as_ref()))
}

pub type Seed = [u8; SEED_SIZE];

#[derive(Debug, Deserialize, Serialize)]
pub struct Admin {
    pub email: String,
    #[serde(
        deserialize_with = "public_key_from_base64",
        serialize_with = "to_base64"
    )]
    pub public_key: PublicKey,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AdminV2 {
    pub email: String,
    #[serde(
        deserialize_with = "public_key_from_dns_safe_base64",
        serialize_with = "public_key_to_dns_safe_base64"
    )]
    pub public_key: PublicKey,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub admin: Admin,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SettingsV2 {
    pub admin: AdminV2,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Config {
    #[serde(rename = "v1")]
    V1 {
        #[serde(deserialize_with = "seed_from_base64", serialize_with = "to_base64")]
        seed: Seed,
        settings: Settings,
    },
    #[serde(rename = "v2")]
    V2 {
        #[serde(deserialize_with = "seed_from_base64", serialize_with = "to_base64")]
        seed: Seed,
        encrypted_key: String,
        registration_code: String,
        settings: SettingsV2,
    },
}

impl Config {
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
            Config::V1 {
                seed,
                settings: Settings { admin },
            },
            holochain_public_key,
        ))
    }

    pub fn new_v2(
        email: String,
        password: String,
        registration_code: String,
        maybe_seed: Option<Seed>,
    ) -> Result<(Self, PublicKey), Error> {
        let seed = match maybe_seed {
            None => OsRng::new()?.gen::<Seed>(),
            Some(s) => s,
        };
        // Eventually this should be the Root bundle/ master bundle
        // that should be returned back to the host
        // So that they can create new keys using that bundle
        let master_secret_key = SecretKey::from_bytes(&seed)?;
        let master_public_key = PublicKey::from(&master_secret_key);

        let admin_keypair = admin_keypair_from(master_public_key, &email, &password)?;
        let admin = AdminV2 {
            email: email.clone(),
            public_key: admin_keypair.public,
        };

        Ok((
            Config::V2 {
                seed,
                encrypted_key: Config::encrypt_key(seed, admin.public_key),
                registration_code,
                settings: SettingsV2 { admin: admin },
            },
            admin_keypair.public,
        ))
    }

    pub fn admin_public_key(&self) -> PublicKey {
        match self {
            Config::V1 { seed: _, settings } => settings.admin.public_key,
            Config::V2 {
                seed: _,
                encrypted_key: _,
                registration_code: _,
                settings,
            } => settings.admin.public_key,
        }
    }

    pub fn encrypt_key(seed: Seed, public_key: PublicKey) -> String {
        // For now lair does not take in any encrypted bytes so we pass back an empty encrypted byte string
        let mut encrypted_key = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ];
        encrypted_key.extend(seed.to_vec());
        encrypted_key.extend(&public_key.to_bytes());
        base64::encode(&encrypted_key)
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

    Ok(Keypair {
        public: public_key,
        secret: secret_key,
    })
}
