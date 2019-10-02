// 
// Manages creation, serialization (storage) and deserialization (loading) of HoloPort Device Seed
// and Admin keys.
// 
// The HoloPort's /var/lib/holo-config.json Config is loaded, and the Config::V1.seed is used to
// derive all Holochain, ZeroTier, etc. keypairs required for operation.  
//
// Admin Keypair
// 
// During creation of the Config, the Admin passphrase is known, and an Admin keypair is derived
// from the entropy collected from the admin email "salt", the holochain Signing public key (Agent
// ID) "pepper", and the Admin passphrase "password".  This yields an HoloPort Admin request signing
// keypair, which is unique to each specific Holoport (due to the email "salt" and Agent ID
// "pepper"), and is secured by the private Admin passphrase.  It can be easily derived (eg. in the
// Admin UI), by knowing the Admin email (eg "admin@example.com"), the target Holoport Agent ID
// (eg. "HcScjabc...123"), and the Admin passphrase (collected from the Admin via the UI).
// 
// We use a temporary Holochain DPKI Keystore to load, parse, validate, save and generate the needed
// Seeds, etc.  This Keystore is never stored, and we secure it in memory using a randomly generated
// password.
// 
// Config Seed Security
// 
// Typical operation of the HoloPort requires unencrypted access to the seed (stored as 24 BIP39
// words containing the 256-bit seed in plaintext), to support unattended rebooting.  This of course
// requires physical and logical security of the removable media containing the serialized Config!
// 
// For those desiring additional security, you can opt to store your HoloPort seed *encrypted*
// (stored as 48 BIP39 words containing the 256-bit seed ciphertext, plus 256-bit Authentication MAC
// and random IV); of course, this requires the HoloPort Admin to log into the HoloPort on every
// reboot, and enter the seed's passphrase to unlock it, and allow the HoloPort to continue booting.
//
use std::fmt;
use arrayref::array_ref;
use ed25519_dalek::*; // PublicKey, ...
use failure::Error;
use rand::{rngs::OsRng, Rng};
use serde::*;

use lib3h_sodium::secbuf::SecBuf;
use holochain_dpki::{
    SEED_SIZE,
    seed, seed::MnemonicableSeed, seed::SeedTrait,
    keypair, keypair::KeyPair
};

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

/*
fn plaintext_seed_from_mnemonic<'de, D>(deserializer: D) -> Result<ConfigSeed, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    String::deserialize(deserializer)
        .and_then(|s| seed::Seed::new_with_mnemonic(s, seed::SeedType::Device).into())
        .map_err(Error::custom)?
}

fn encrypted_seed_from_mnemonic<'de, D>(deserializer: D) -> Result<ConfigSeed, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    String::deserialize(deserializer)
        .and_then(|s| seed::EncryptedSeed::new_with_mnemonic(s, seed::SeedType::Device).into())
        .map_err(Error::custom)?
}
 */

const ARGON2_CONFIG: argon2::Config = argon2::Config {
    variant: argon2::Variant::Argon2id,
    version: argon2::Version::Version13,
    mem_cost: 1 << 16, // 64 MB
    time_cost: 2,
    lanes: 4,
    thread_mode: argon2::ThreadMode::Parallel,
    secret: &[],
    ad: b"holo-config admin ed25519 key v1",
    hash_length: SECRET_KEY_LENGTH as u32,
};

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

enum ConfigSeed {
    PlaintextSeed(seed::DeviceSeed),
    EncryptedSeed(seed::EncryptedSeed),
}


impl fmt::Display for ConfigSeed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            ConfigSeed::EncryptedSeed(es) => s.get_mnemonic(),
            ConfigSeed::PlaintextSeed(s) => s.seed().get_mnemonic(),
        }.map_err(|_| std::fmt::Error)?)
    }
}

impl fmt::Debug for ConfigSeed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Serialize for ConfigSeed {
    /// Output ConfigSeed::Encrypted/PlaintextSeeds as mnemonics, via standard Display format.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

impl<'d> Deserialize<'d> for ConfigSeed {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'d>,
    {
        use serde::de::Error;
        let mnemonic = String::deserialize(deserializer)?;
        if let Ok(es) = seed::EncryptedSeed::new_with_mnemonic(mnemonic.clone(), seed::SeedType::Device) {
            Ok(ConfigSeed::EncryptedSeed(es))
        } else {
            match seed::Seed::new_with_mnemonic(mnemonic, seed::SeedType::Device) {
                Ok(s) => Ok(ConfigSeed::PlaintextSeed(seed::DeviceSeed::new(s.buf))),
                Err(e) => Err(Error::custom(e)),
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Config {
    #[serde(rename = "v1")]
    V1 {
        seed: ConfigSeed,
        admins: Vec<Admin>,
    },
}

impl Config {
    /// Use an email/password and (optionally) 256-bit seed bytes to generate a Config.  The
    /// resultant Config will contain a 256-bit seed in BIP39 mnemonic form; either in plaintext (24
    /// words), or ciphertext + MAC/IV Tag (48 words).  While this seed may have been generated in a
    /// variety of ways (eg. randomly, or via DPKI), it will be stored in this Config as a
    /// DeviceSeed (24 words) or (optionally) an EncryptedSeed (48 words), secured by the supplied
    /// Admin password.
    /// 
    /// Returns the Config, and the Holochain Agent ID.
    pub fn new(
        email: String,
        password: String,
        maybe_seed: Option<Seed>,
        encrypt: bool
    ) -> Result<(Self, String), Error> {
        // Get the seed bytes, either generated from OS random source, or from the supplied seed.
        let seed_bytes = match maybe_seed {
            None => OsRng::new()?.gen::<Seed>(),
            Some(s) => s,
        };
        
        // Construct the Holochain Signing Public Key from the seed; this calls the holochain
        // conductor API, via the Keystore, to (eventually) invoke holochain_dpki's
        // keypair::SigningKeyPair.new_from_seed directly with the supplied seed.  This seed might
        // be a seed::OneShot seed, created "ex nihilo" for this HoloPort.  Or, it might be a
        // seed::DeviceSeed, derived by index from a seed;:RootSeed.  But, finally, this seed
        // material is directly used by holochain_dpki::keypair::SigningKeyPair.new_from_seed.
        //let (_, holochain_public_key) = keystore::from_seed(&seed)?;
        let mut seed_buf = SecBuf::with_insecure(SEED_SIZE);
        seed_buf.from_array(&seed_bytes)?;
        let holochain_signing_keypair = keypair::SigningKeyPair::new_from_seed(&mut seed_buf)?;

        let admin = Admin {
            email: email.clone(),
            public_key: admin_public_key_from(
                &holochain_signing_keypair, &email, &password)?,
        };

        let seed_dev = seed::DeviceSeed::new(seed_buf);
        Ok((
            Config::V1 {
                admins: vec![admin],
                seed: if encrypt {
                    ConfigSeed::EncryptedSeed(seed_dev.encrypt(password, None)?)
                } else {
                    ConfigSeed::PlaintextSeed(seed_dev)
                }
            },
            holochain_signing_keypair.public(), // eg. "HcScja1..Z0"
        ))
    }
}

fn admin_public_key_from(
    holochain_public_key: &keypair::SigningKeyPair,
    email: &str,
    password: &str,
) -> Result<PublicKey, Error> {
    // This allows to use email addresses shorter than 8 bytes.
    let salt = Sha512::digest(email.as_bytes());

    let mut config = ARGON2_CONFIG.clone();
    config.secret = &holochain_public_key.decode_pub_key();

    let hash = &argon2::hash_raw(&password.as_bytes(), &salt, &config)?;

    Ok(PublicKey::from(&SecretKey::from_bytes(hash)?))
}
