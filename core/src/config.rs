use arrayref::array_ref;
use ed25519_dalek::{Digest, Sha512, SigningKey, VerifyingKey};
use failure::Error;
use rand::{rngs::OsRng, Rng};
use serde::*;
pub const SEED_SIZE: usize = 32;

fn public_key_from_base64<'de, D>(deserializer: D) -> Result<VerifyingKey, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)
        .and_then(|s| {
            base64::decode_config(s, base64::STANDARD_NO_PAD)
                .map_err(|err| de::Error::custom(err.to_string()))
        })
        .map(|bytes| match bytes[0..32].try_into() {
            Ok(b) => VerifyingKey::from_bytes(&b).map_err(|e| e.to_string()),
            Err(_) => Err("Public key is not 32 bytes long".to_string()),
        })
        .and_then(|maybe_key| maybe_key.map_err(|err| de::Error::custom(err.to_string())))
}

pub fn seed_from_base64<'de, D>(deserializer: D) -> Result<Seed, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)
        .and_then(|s| base64::decode(s).map_err(|err| de::Error::custom(err.to_string())))
        .map(|bytes| *array_ref!(bytes, 0, SEED_SIZE))
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
    pub public_key: VerifyingKey,
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
        /// This is the Device Seed Bundle as a base64 string which is compatible with lair-keystore >=v0.0.8
        device_bundle: String,
        /// Derivation path of the seed in this config that was generated for a Master Seed
        derivation_path: String,
        /// Holo registration code is used to identify and authenticate its users
        registration_code: String,
        /// The pub-key in settings is the holoport key that is used for verifying login signatures
        settings: Settings,
    },
    #[serde(rename = "v3")]
    V3 {
        /// This is the Device Seed Bundle as a base64 string which is compatible with lair-keystore >=v0.0.8
        /// And is encoded with a password that will be needed to be used to decrypt it
        device_bundle: String,
        // The revocation key is usually the /0 derivation path of the master seed
        revocation_pub_key: String,
        // /1 derivation path of the device bundle
        holoport_id: String,
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
    ) -> Result<(Self, VerifyingKey), Error> {
        let (seed, admin_keypair, holochain_public_key) =
            generate_keypair(email.clone(), password, maybe_seed)?;
        let admin = Admin {
            email,
            public_key: admin_keypair.verifying_key(),
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
        device_bundle: String,
        device_pub_key: VerifyingKey,
    ) -> Result<(Self, VerifyingKey), Error> {
        let admin_keypair = admin_keypair_from(device_pub_key, &email, &password)?;
        let admin = Admin {
            email,
            public_key: admin_keypair.verifying_key(),
        };
        Ok((
            Config::V2 {
                device_bundle,
                derivation_path,
                registration_code,
                settings: Settings { admin },
            },
            device_pub_key,
        ))
    }

    pub fn admin_public_key(&self) -> VerifyingKey {
        match self {
            Config::V1 { settings, .. } | Config::V2 { settings, .. } => settings.admin.public_key,
        }
    }
}

fn generate_keypair(
    email: String,
    password: String,
    maybe_seed: Option<Seed>,
) -> Result<(Seed, SigningKey, VerifyingKey), Error> {
    let master_seed = match maybe_seed {
        None => OsRng::new()?.gen::<Seed>(),
        Some(s) => s,
    };
    let master_secret_key = SigningKey::from_bytes(&master_seed);
    let master_public_key = VerifyingKey::from(&master_secret_key);

    let admin_keypair = admin_keypair_from(master_public_key, &email, &password)?;
    Ok((master_seed, admin_keypair, master_public_key))
}

pub fn admin_keypair_from(
    holochain_public_key: VerifyingKey,
    email: &str,
    password: &str,
) -> Result<SigningKey, Error> {
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

    Ok(SigningKey::from_bytes(&hash))
}
