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

const ARGON2_ADDITIONAL_DATA: &[u8] = b"holo-config admin ed25519 key v1";

pub type Seed = [u8; SEED_SIZE];

#[derive(Debug, Deserialize, Serialize)]
pub struct Admin {
    email: String,
    #[serde(
        deserialize_with = "public_key_from_base64",
        serialize_with = "to_base64"
    )]
    public_key: PublicKey,
    device_name: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InternalConfig {
    admin: Admin
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Config {
    #[serde(rename = "v1")]
    V1 {
        #[serde(deserialize_with = "seed_from_base64", serialize_with = "to_base64")]
        seed: Seed,
        config: InternalConfig,
    },
}

impl Config {
    pub fn new(
        email: String,
        password: String,
        device_name: String,
        maybe_seed: Option<Seed>,
    ) -> Result<(Self, PublicKey), Error> {
        let seed = match maybe_seed {
            None => OsRng::new()?.gen::<Seed>(),
            Some(s) => s,
        };

        let holochain_secret_key = SecretKey::from_bytes(&seed)?;
        let holochain_public_key = PublicKey::from(&holochain_secret_key);

        let admin = Admin {
            email: email.clone(),
            public_key: admin_public_key_from(holochain_public_key, &email, &password)?,
            device_name: device_name.clone()
        };

        Ok((
            Config::V1 {
                config: InternalConfig {
                    admin: admin
                },
                seed: seed,
            },
            holochain_public_key,
        ))
    }
}

fn admin_public_key_from(
    holochain_public_key: PublicKey,
    email: &str,
    password: &str,
) -> Result<PublicKey, Error> {
    // This allows to use email addresses shorter than 8 bytes.
    let salt = Sha512::digest(email.as_bytes());
    let mut hash = [0; SEED_SIZE];

    argon2min::Argon2::new(2, 4, 1 << 16, argon2min::Variant::Argon2id)?
        .hash(&mut hash, password.as_bytes(), &salt, &holochain_public_key.to_bytes(), ARGON2_ADDITIONAL_DATA);

    Ok(PublicKey::from(&SecretKey::from_bytes(&hash)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_public_key() {
        let email: String = "pj@aa.pl".to_string();
        let password: String =  "password".to_string();
	let device_name: String = "MyHoloport".to_string();
        let seed: [u8; 32] = [55; 32];
        let expected_public_key: [u8; 32] = [17, 243, 42, 222, 75, 47, 128, 87, 1, 252, 72, 56, 141, 216, 210, 251, 217, 95, 97, 62, 95, 112, 234, 31, 243, 73, 64, 160, 134, 92, 138, 97];

        let (config, _) = Config::new(email, password, device_name, Some(seed)).unwrap();
        if let Config::V1{seed: _, config} = config {
            assert_eq!(config.admin.public_key.to_bytes(), expected_public_key);
        } else {
            panic!("Wrong Version of Config implementation");
        }
    }
}
