use arrayref::array_ref;
use ed25519_dalek::*;
use failure::Error;
use hcid::HcidEncoding;
use lazy_static::lazy_static;
use rand::{rngs::OsRng, Rng};
use serde::*;
use std::fmt;
use std::str::FromStr;


const SEED_SIZE: usize = 32;


/// Admin... keys are just ed25591 signing keys w/ some different serialization; HcAc...
#[derive(Debug, Clone)]
pub struct AdminPublicKey(pub PublicKey);

lazy_static! {
    pub static ref HCAPK_CODEC: hcid::HcidEncoding =
        HcidEncoding::with_kind("hca0").expect("Couldn't init hca0 hcid codec.");
}

impl AdminPublicKey {
    #[inline]
    pub fn to_bytes(&self) -> [u8; PUBLIC_KEY_LENGTH] {
        self.0.to_bytes()
    }

    pub fn verify(
        &self,
        message: &[u8],
        signature: &Signature,
    ) -> bool {
        self.0.verify(message, signature).is_ok()
    }
}

impl fmt::Display for AdminPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", HCAPK_CODEC.encode(self.0.as_bytes())
               .map_err(|_e| std::fmt::Error)?)
    }
}

impl FromStr for AdminPublicKey {
    type Err = Error;
    fn from_str(hcapk: &str) -> Result<Self, Self::Err> {
        Ok(AdminPublicKey(
            PublicKey::from_bytes(
                &HCAPK_CODEC.decode(hcapk)?
            )?
        ))
    }
}

impl Serialize for AdminPublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

impl<'d> Deserialize<'d> for AdminPublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'d>,
    {
        let hcapk = String::deserialize(deserializer)?; // HcA...
        Ok(
            AdminPublicKey::from_str(&hcapk)
                .map_err(de::Error::custom)?
        )
    }
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
    public_key: AdminPublicKey,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Config {
    #[serde(rename = "v1")]
    V1 {
        #[serde(deserialize_with = "seed_from_base64", serialize_with = "to_base64")]
        seed: Seed,
        admin: Admin,
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
            public_key: AdminPublicKey(admin_keypair.public),
        };

        Ok((
            Config::V1 {
                admin: admin,
                seed: seed,
            },
            holochain_public_key,
        ))
    }
}

/// Generate a Holo admin keypair from holochain public key, email and password
pub fn admin_keypair_from(
    holochain_public_key: PublicKey,
    email: &str,
    password: &str,
) -> Result<Keypair, Error> {
    // This allows to use email addresses shorter than 8 bytes.
    let salt = Sha512::digest(email.as_bytes());
    keypair_from(
        password.as_bytes(),
        &salt,
        &holochain_public_key.to_bytes(),
        ARGON2_ADDITIONAL_DATA
    )
}

/// Most general function for generating Holo keypairs. 
/// Prefer using a more specialised version (e.g. admin_keypair) if possible
/// passphrase: bytes of a user readable passphrase to be hashed when deriving the key pair
/// salt: Addtional salt to be added as part of the Argon2 algo
/// extra_secret: Optional, pass empty byte array if not desired. See argon2min docs
/// argon_additional_data: Optional, pass empty byte array if not desired. See argon2min docs
pub fn keypair_from(
    passphrase: &[u8],
    salt: &[u8],
    extra_secret: &[u8],
    argon_additional_data: &[u8]
) -> Result<Keypair, Error> {
    let mut hash = [0; SEED_SIZE];
    argon2min::Argon2::new(2, 4, 1 << 16, argon2min::Variant::Argon2id)?
        .hash(&mut hash, passphrase, &salt, &extra_secret, argon_additional_data);
    let secret_key = SecretKey::from_bytes(&hash)?;
    let public_key = PublicKey::from(&secret_key);
    Ok(Keypair{
        public: public_key,
        secret: secret_key,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_public_key() {
        let email: String = "pj@aa.pl".to_string();
        let password: String =  "password".to_string();
        let seed: Option<[u8; 32]> = Some([55; 32]);
        let expected_public_key: [u8; 32] = [17, 243, 42, 222, 75, 47, 128, 87, 1, 252, 72, 56, 141, 216, 210, 251, 217, 95, 97, 62, 95, 112, 234, 31, 243, 73, 64, 160, 134, 92, 138, 97];

        let (config, _) = Config::new(email, password, seed).unwrap();
        let Config::V1 { admin, .. } = &config;
        assert_eq!( admin.public_key.to_bytes(), expected_public_key );
        // Ensure that JSON serialization works as expected
        assert_eq!( serde_json::to_string_pretty( &config ).unwrap(), r#"{
  "v1": {
    "seed": "Nzc3Nzc3Nzc3Nzc3Nzc3Nzc3Nzc3Nzc3Nzc3Nzc3Nzc",
    "admin": {
      "email": "pj@aa.pl",
      "public_key": "HcAcIEqufMqexm6ak6a9zTbzsynof883m7ru6Y5r7iq9gTkaVcDF3cUBAdba4jz"
    }
  }
}"# );

    }
}
