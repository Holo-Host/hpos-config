use arrayref::array_ref;
use ed25519_dalek::*;
use failure::Error;
use rand::{rngs::OsRng, Rng};
use serde::*;
pub const SEED_SIZE: usize = 32;

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

pub fn seed_from_base64<'de, D>(deserializer: D) -> Result<Seed, D::Error>
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

const ARGON2_ADDITIONAL_DATA: &[u8] = b"hpos-config admin ed25519 key v1";

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
pub struct Settings {
    pub admin: Admin,
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
        // This is the Device Seed Bundle which is compatible with lair-keystore >=v0.0.8
        #[serde(deserialize_with = "seed_from_base64", serialize_with = "to_base64")]
        seed: Seed,
        /// Derivation path of the seed in this config that was generated for a Master Seed
        derivation_path: String,
        /// Holo registration code is used to identify and authenticate its users
        registration_code: String,
        /// The pub-key in settings is the holoport key that is used for verifying login signatures
        settings: Settings,
    },
}

impl Config {
    pub fn new(
        email: String,
        password: String,
        maybe_seed: Option<Seed>,
    ) -> Result<(Self, PublicKey), Error> {
        let (seed, admin_keypair, holochain_public_key) =
            generate_keypair(email.clone(), password, maybe_seed)?;
        let admin = Admin {
            email: email,
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
        derivation_path: String,
        device_seed: Option<Seed>,
    ) -> Result<(Self, PublicKey), Error> {
        let (device_master_seed, admin_keypair, hp_id_pub_key) =
            generate_keypair(email.clone(), password, device_seed)?;
        let admin = Admin {
            email: email,
            public_key: admin_keypair.public,
        };
        Ok((
            Config::V2 {
                seed: device_master_seed,
                derivation_path,
                registration_code,
                settings: Settings { admin: admin },
            },
            hp_id_pub_key,
        ))
    }

    pub fn admin_public_key(&self) -> PublicKey {
        match self {
            Config::V1 { settings, .. } | Config::V2 { settings, .. } => settings.admin.public_key,
        }
    }

    pub fn holoport_public_key(&self) -> Result<PublicKey, Error> {
        match self {
            Config::V1 { seed, .. } | Config::V2 { seed, .. } => {
                let secret_key = SecretKey::from_bytes(seed)?;
                Ok(PublicKey::from(&secret_key))
            }
        }
    }

    pub fn encoded_ed25519_keypair(&self) -> Result<String, Error> {
        match self {
            Config::V1 { seed, .. } | Config::V2 { seed, .. } => {
                let secret_key = SecretKey::from_bytes(seed)?;
                Ok(Config::encrypt_key(seed, PublicKey::from(&secret_key)))
            }
        }
    }

    pub fn decoded_to_ed25519_keypair(blob: &String) -> Result<Keypair, Error> {
        let decoded_key = base64::decode(blob)?;
        Ok(Keypair {
            public: PublicKey::from_bytes(&decoded_key[32..64].to_vec())?,
            secret: SecretKey::from_bytes(&decoded_key[64..].to_vec())?,
        })
    }

    pub fn encrypt_key(seed: &Seed, public_key: PublicKey) -> String {
        // For now lair does not take in any encrypted bytes so we pass back an empty encrypted byte string
        let mut encrypted_key = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ];
        encrypted_key.extend(&public_key.to_bytes());
        encrypted_key.extend(seed.to_vec());
        base64::encode(&encrypted_key)
    }
}

fn generate_keypair(
    email: String,
    password: String,
    maybe_seed: Option<Seed>,
) -> Result<(Seed, Keypair, PublicKey), Error> {
    let master_seed = match maybe_seed {
        None => OsRng::new()?.gen::<Seed>(),
        Some(s) => s,
    };
    let master_secret_key = SecretKey::from_bytes(&master_seed)?;
    let master_public_key = PublicKey::from(&master_secret_key);

    let admin_keypair = admin_keypair_from(master_public_key, &email, &password)?;
    Ok((master_seed, admin_keypair, master_public_key))
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
