// 
// Manages creation, serialization (storage) and deserialization (loading) of HoloPort Device Seed
// and Admin keys.
// 
// HoloPort Usage
// 
// - Loads seed
//   - May be encrypted, which prevents further keypair derivation until the HP Admin queries the
//     seed, decrypts it, generates all Agent and Admin keypairs, and returns them to the HoloPort.
//     - Instead of running the 
//   - With 

// 
// - Generates Agent ID public (HcScJ123abc...) keypair.  No key material is persisted to the HoloPort
//   disk; without the removable media containing the seed, an attacker cannot sign entries into any
//   of the HoloPort's hApp's source-chains.
// 
//   - Other keypairs, eg ZeroTier, SSH Host keys, could be generated, but will probably be left as
//     automatically generated local HoloPort state, because sometimes ZeroTier may need to
//     regenerate keys and does so automatically (eg. on collision of 40-bit zerotier device IDs).
//     At 1 in a Trillion, the probability appears low -- but this could be an attack vector.
//     Unintuitively, with 1 million HoloPorts, the probability of such a collision reaches 40%!
//     So, deterministic key generation would require testing for such collisions, and generating
//     further keys.  Best to let ZeroTier do it; especially since the ZT IP address of the host is
//     dynamically updated on a regular basis to avoid an external persistent database of ZT IPs.
// 
//        people = 1000000                                         = 1,000,000
//        days = 1000000000000                                     = 1,000,000,000,000
//        pairs = ((people)*(people-1)) / 2                        = 499,999,500,000
//        chance per pair = ((days-1) / days) ^ pairs              = 0.60653767176
//        percent chance of all different = chance per pair * 100  = 60.65376717624
// 
// 
// The HoloPort's holo-config.json Config is loaded (probably from a very temporarily mounted USB
// device, early in the boot), and the Config::V1.seed is used to derive all the original Holochain
// (Agent ID) keypair, ZeroTier keys, and Admin etc. keypairs required for operation.
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Admin {
    email: String,
    public_key: AdminSigningPublicKey,
}

// Seeds will be represented by either 24 or 48 words
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum ConfigSeed {
    PlaintextSeed(Seed),
    EncryptedSeed(EncryptedSeed),
}

impl fmt::Display for ConfigSeed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigSeed::PlaintextSeed(s) => write!(f, "{}", s),
            ConfigSeed::EncryptedSeed(s) => write!(f, "{}", s),
        }
    }
}

/// The HoloPort Config data saved into holo-config.json.  From this seed, all Agent ID keypair and
/// Admin keypair(s) can be derived.  If desired, the Seed may be encrypted with the admin password,
/// thereby preventing an attacker who recovers this file from easily obtaining the 
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Config {
    #[serde(rename = "v1")]
    V1 {
        seed: ConfigSeed,
        agent_id: SigningPublicKey,
        admins: Vec<Admin>,
    },
}

/// The Config, and the full set of derived keys, including signing private keys
#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigResult {
    pub config: Config,
    pub agent_key: SigningSecretKey,
    pub admin_keys: Vec<SigningSecretKey>,
}

impl Config {
    /// Use an email/password and (optionally) 256-bit seed bytes to generate a Config, and all
    /// derived keypairs.  The resultant Config will contain a 256-bit seed in BIP39 mnemonic form;
    /// either in plaintext (24 words), or ciphertext + MAC/IV Tag (48 words).  While this seed may
    /// have been generated in a variety of ways (eg. randomly, or via DPKI), it will be stored in
    /// this Config as a DeviceSeed (24 words) or (optionally) an EncryptedSeed (48 words) secured
    /// by the supplied Admin password.
    /// 
    /// Returns the Config, and the Holochain Agent ID for which this Config was created, and the
    /// Vec<SigningSecretKey> matching the `admins` Vec of email and public_key values.
    pub fn new(
        email: String,
        password: String,
        maybe_seed: Option<ConfigSeed>, // None: generate, Some(...): optionally decrypt, and use
        encrypt: bool // Encrypt the resultant Config's seed
    ) -> Result<ConfigResult, Error> {
        // Get the Seed bytes, either generated from OS random source, or from the supplied data.
        let seed_secret: Seed = match maybe_seed {
            None => Seed::from_bytes(&OsRng::new()?.gen::<[u8; SEED_SIZE]>())?,
            Some(ConfigSeed::EncryptedSeed(s)) => s.decrypt(&password)?,
            Some(ConfigSeed::PlaintextSeed(s)) => s,
        };
        
        // Construct the Holochain Agent ID Signing Public Key from the seed; holochain DPKI
        // directly generates an ed25519 Signing keypair from the seed entropy.
        let (agent_id, agent_key) = signing_keypair_from_seed(seed_secret.as_bytes())?;

        // Then, we generate Admin keys password entropy, salted by email, and peppered (made
        // unique) by the Agent ID public key.  This allows a user to specify the same (!)
        // email+password for multiple HoloPort Admin, and get unique Admin keys for each one.
        let (admin_pubkey, admin_key) = admin_keypair_from(
            &agent_id, &email, &password
        )?;

        // Now, (re) generate the Config w/ the desired seed encryption
        let seed = if encrypt {
            ConfigSeed::EncryptedSeed(seed_secret.encrypt(&password)?)
        } else {
            ConfigSeed::PlaintextSeed(seed_secret)
        };
        let admins = vec![Admin {
                email: email.clone(),
                public_key: admin_pubkey,
        }];

        // And package it up with the derived secret keys
        let config = Config::V1 { seed, agent_id, admins };
        let admin_keys = vec![
            admin_key
        ];
        Ok(ConfigResult { config, agent_key, admin_keys })
    }
}

pub fn admin_keypair_from(
    holochain_pubkey: &SigningPublicKey,
    email: &str,
    password: &str,
) -> Result<(AdminSigningPublicKey, SigningSecretKey), Error> {
    let seed = email_password_to_seed(
        email, password, Some(holochain_pubkey.as_bytes()),
    )?;
    let admin_secret = SigningSecretKey::from_bytes(seed.as_bytes())?;
    let admin_pubkey = AdminSigningPublicKey(SigningPublicKey::from(&admin_secret));
    Ok((admin_pubkey, admin_secret))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_round_trip() {
        let ConfigResult { config, .. } = Config::new(
            "a@b.c".to_string(),
            "password".to_string(),
            Some(ConfigSeed::PlaintextSeed(Seed::from_bytes(&[0u8; 32 ]).unwrap())),
            false
        ).unwrap();
        
        let Config::V1{
            seed: origin_seed, agent_id: origin_agent_id, admins: _origin_admins
        } = config.clone();
	assert_eq!(origin_agent_id.to_string(), "HcSCIp5KE88N7OwefwsKhKgRfJyr465fgikyphqCIpudwrcivgfWuxSju9mecor");
        assert_eq!(format!("{}", &origin_seed),
                   "abandon abandon abandon abandon abandon abandon abandon abandon \
		    abandon abandon abandon abandon abandon abandon abandon abandon \
		    abandon abandon abandon abandon abandon abandon abandon art");

        let s: String = serde_json::to_string(&config).unwrap();

        let Config::V1{ seed: config_seed, .. } = serde_json::from_str(&s).unwrap();
        assert_eq!(format!("{}", &config_seed), format!("{}", &origin_seed));
    }
}
