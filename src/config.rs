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
// We use a simple set of DPKI-compatible APIs to load, parse, validate, save and generate the needed
// Seeds, etc.
// 
// Config Seed Security
// 
// Typical operation of the HoloPort requires unencrypted access to the seed (stored as 24 BIP39
// words containing the 256-bit seed in plaintext), to support unattended rebooting.  This of course
// requires physical and logical security of the removable media containing the serialized Config!
// 
// For those desiring additional security, you can opt to store your HoloPort seed *encrypted*
// (stored as 48 BIP39 words containing the 256-bit seed ciphertext, plus 256-bit Authentication MAC
// and random salt); of course, this requires the HoloPort Admin to log into the HoloPort on every
// reboot, and enter the seed's passphrase to unlock it, and allow the HoloPort to continue booting.
//
use std::fmt;
use failure::Error;
use hcid;
use rand::{
    rngs::OsRng, Rng
};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer, de
};

use crate::dpki:: {
    SEED_SIZE,
    Seed, EncryptedSeed, SeedData,
    SigningPublicKey, SigningSecretKey,
    email_password_to_seed, signing_keypair_from_seed,
};

// Admin... keys are just signing keys w/ some different serialization; HcAc... instead of HcSc....
#[derive(Debug)]
pub struct AdminSigningPublicKey(pub SigningPublicKey);

impl fmt::Display for AdminSigningPublicKey {
     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
         write!(f, "{}", hcid::HcidEncoding::with_kind("hca0")
                .map_err(|_e| std::fmt::Error)?
                .encode((self.0).0.as_bytes())
                .map_err(|_e| std::fmt::Error)?)
    }
}

impl Serialize for AdminSigningPublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

impl<'d> Deserialize<'d> for AdminSigningPublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'d>,
    {
        let hcid = String::deserialize(deserializer)?;
        Ok(AdminSigningPublicKey(
            SigningPublicKey::from_bytes(
                &hcid::HcidEncoding::with_kind("hca0")
                    .map_err(de::Error::custom)?
                    .decode(&hcid)
                    .map_err(de::Error::custom)?
            ).map_err(de::Error::custom)?
        ))
    }
}

// Config packages up the seed entropy used to generate the Holochain Signing keypair, and the
// derived Admin keys, used to authenticate HoloPort admin requests.
#[derive(Debug, Deserialize, Serialize)]
pub struct Admin {
    email: String,
    public_key: AdminSigningPublicKey,
}

// Seeds will be represented by either 24 or 48 words
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ConfigSeed {
    PlaintextSeed(Seed),
    EncryptedSeed(EncryptedSeed),
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
    ) -> Result<(Self, SigningPublicKey), Error> {
        // Get the seed bytes, either generated from OS random source, or from the supplied seed.
        let seed = match maybe_seed {
            None => Seed::from_bytes(&OsRng::new()?.gen::<[u8; SEED_SIZE]>())?,
            Some(s) => s,
        };
        
        // Construct the Holochain Signing Public Key from the seed; holochain DPKI directly
        // generates an ed25519 Signing keypair from the seed entropy.
        let (holochain_pubkey, _) = signing_keypair_from_seed(seed.as_bytes())?;

        let admin_public_key = admin_public_key_from(
            &holochain_pubkey, &email, &password
        )?;
                                              
        let admin = Admin {
            email: email.clone(),
            public_key: admin_public_key,
        };
        let config_seed = if encrypt {
            ConfigSeed::EncryptedSeed(seed.encrypt(&password)?)
        } else {
            ConfigSeed::PlaintextSeed(seed)
        };
        Ok((
            Config::V1 {
                admins: vec![admin],
                seed: config_seed,
            },
            holochain_pubkey,
        ))
    }
}

pub fn admin_public_key_from(
    holochain_pubkey: &SigningPublicKey,
    email: &str,
    password: &str,
) -> Result<AdminSigningPublicKey, Error> {
    let seed = email_password_to_seed(
        email, password, Some(holochain_pubkey.as_bytes()),
    )?;
    Ok(AdminSigningPublicKey(
        SigningPublicKey::from(
            &SigningSecretKey::from_bytes(seed.as_bytes())?
        )
    ))
}
