

use ed25519_dalek::*;
//use rand::{Rng, RngCore, CryptoRng};
use rand;
use std::time::{Duration, SystemTime};
use crypto::{
    //aead::AeadEncryptor, chacha20poly1305::ChaCha20Poly1305,
    aes, blockmodes,
    buffer::{ ReadBuffer, WriteBuffer, BufferResult, RefReadBuffer, RefWriteBuffer },
    sha2, digest::Digest,
};

use crate::error::*;

// See: lib3h/crates/sodium/src/pwhash.rs, crypto_pwhash_argon2id_{OPS,MEM}LIMIT_INTERACTIVE
pub const OPSLIMIT_INTERACTIVE: u32 = 2;
pub const MEMLIMIT_INTERACTIVE: u32 = 1 << 16; // ~67MB
pub const HASHBYTES: u32 = 32;
pub const KEYBYTES: usize = 32;
pub const AEAD_TAGBYTES: usize = 16; // AEAD encryption/authentication tag

pub const HOLO_ADMIN_ARGON_CONFIG: argon2::Config = argon2::Config {
    variant:	argon2::Variant::Argon2id,
    version:	argon2::Version::Version13,
    mem_cost:	MEMLIMIT_INTERACTIVE,
    time_cost:  OPSLIMIT_INTERACTIVE,
    lanes:	4,
    thread_mode: argon2::ThreadMode::Parallel,
    secret:	&[],
    ad:		&[],
    hash_length: HASHBYTES,
};

pub const HOLO_ENTROPY_SIZE:	usize = 32;

/// The collection of data required to configure a HoloPort.
#[derive(Deserialize, Debug, Serialize)]
pub struct HoloPortConfiguration {
    // All admin requests are signed with the private key, computed from the HoloPort owner's email
    // address (as salt) and password; authenticate requests 
    name: 		Option<String>, // A unique name for the HoloPort (if any); hashed w/ password
    email:		String, // HoloPort admin/owner email; used as salt for argon2 password
    admin_pubkey:	String, // All Admin API requests are signed by the admin key
    
    // The cryptographic seed entropy supplied, from which all HoloPort Agent and ZeroTier
    // public/private keys are derived.  This deterministicaly defines the Agent ID, ZeroTier
    // network ID, etc. of this HoloPort.  We confirm that it is correct, by ensuring that the
    // resultant seed entropy was encrypted and signed by an AEAD private key derived from the the
    // admin key.
    //
    // We *explicitly* support a configuration with the seed entropy shipped *in the clear*, with
    // the decryption/authentication key; yes, this means that physical access to the HoloPort, or
    // logical access to the storage containing this configuration, yields the Holo Agent and
    // ZeroTier public/private keys!
    // 
    // The seed is *always* encrypted with a separate "blinding" AEAD keypair derived from the
    // "admin" key (it can only be encrypted by the admin).  This use-case is explicitly supported
    // by supplying the optional seed_pubkey decryption/authentication key, here.
    // 
    // If the seed "blinding" key is *not* supplied, then the HoloPort cannot proceed with Holo
    // start-up until the seed entropy is decrypted by the HoloPort admin.  This AEAD keypair is
    // derived from the admin private key, and both encrypts and authenticates the seed material.
    // This is more secure, but also means that unattended HoloPort boot is not supported...
    seed_key:		Option<String>, // 256-bit AES ECB "blinding" encryption key
    seed:		String, // The base-64 encoded AEAD tag + seed used to generate all IDs
    seed_sig:		String, // Signed by admin private key
}

impl std::fmt::Display for HoloPortConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "name: {:?}, email: {}, admin_pubkey: {}, seed_key: {:?}, seed: {}, seed_sig: {}",
               &self.name, &self.email, &self.admin_pubkey,
               &self.seed_key, &self.seed,  &self.seed_sig,
        )
    }
}

impl HoloPortConfiguration {
    /// new -- Create a new config from provided email/password, + optional seed entropy
    /// 
    /// Deduces and creates the admin and seed blinding keys, and creates the config.
    pub fn new(
        name_maybe:	Option<String>,
        email:		String,
        password:	String,
        seed_maybe:	Option<[u8; HOLO_ENTROPY_SIZE]>
    ) -> Result<Self, ConfigurationError> {
        let seed_entropy = match seed_maybe {
            Some(s) => s,
            None => {
                let out: [u8; HOLO_ENTROPY_SIZE] = rand::random();
                out
            },
        };
        let admin_keypair = admin_key_from(&email, &password, &name_maybe)?;

        // The raw seed entropy is signed by the admin private key, so we can ensure it's not
        // corrupt, or improperly decrypted.
        let seed_sig = admin_keypair.sign(&seed_entropy);

        let blind_keypair = blind_key_from(&admin_keypair.secret.as_bytes())?;

        /*
         * AEAD gives  us no benefit, as we cannot ensure that the decryption key provided 
         * was actually the one derived from the admin private key.
         * 
        // Always encrypt the seed w/ the derived "blinding" key; the AEAD "tag" (Nonce, MAC and
        // size) is prefixed to the resultant encrypted seed.
        let seed_nonce: [u8; 8] = rand::random();
        let seed_aad = [0u8; 0]; // additional authenticated data
        let seed_key = blind_keypair.secret.to_bytes();
        let mut seed_encrypted = [0u8; HOLO_ENTROPY_SIZE];
        let mut seed_tag = [0u8; AEAD_TAGBYTES];
        ChaCha20Poly1305::new(&seed_key, &seed_nonce, &seed_aad)
            .encrypt(&seed_entropy, &mut seed_encrypted, &mut seed_tag);
        let mut seed_cipher = [0u8; AEAD_TAGBYTES + HOLO_ENTROPY_SIZE];
        *array_mut_ref![seed_cipher, 0, AEAD_TAGBYTES] = *array_ref![seed_tag, 0, AEAD_TAGBYTES];
        *array_mut_ref![seed_cipher, AEAD_TAGBYTES, HOLO_ENTROPY_SIZE] = *array_ref![seed_encrypted, 0, HOLO_ENTROPY_SIZE];
         * 
         */
        // To use AES CBC, we could use the admin public key as the IV.  The admin keypair, the
        // derived blinding key, and the seed entropy are all created simultaneously, and the
        // blinding key is used exactly once, so the requirement that the AES CBC IV be
        // "unpredictable prior to use" is satisfied.  We don't have to come up with subsequent IVs,
        // because we are using the key only one time.  We can also use AES ECB w/ no IV, because we
        // are producing exactly one cipher-block of output.
        let seed_key = blind_keypair.secret.to_bytes();
        let seed_cipher = encrypt_seed_with(&seed_entropy, &seed_key)?;

        let config = HoloPortConfiguration {
            name:		name_maybe.to_owned(),
            email:		email.to_string(),
            admin_pubkey:	hcid::HcidEncoding::with_kind("hca0")?.encode(&admin_keypair.public.to_bytes())?,
            seed:		hcid::HcidEncoding::with_kind("hcc0")?.encode(&seed_cipher)?,
            seed_sig:		base64::encode(&seed_sig.to_bytes().into_iter()),
            seed_key:		Some(hcid::HcidEncoding::with_kind("hcb0")?.encode(&seed_key)?),
        };

        // Final test to guarantee round-trip correctness
        if decrypt_seed_with(
            &config.admin_pubkey,		// HcA...  Holo Admin key
            &config.seed_key.clone().unwrap(),	// HcB...  Holo Blinding key
            &config.seed,			// HcC...  Holo Config seed (Encrypted w/ Blinding key)
            &config.seed_sig,			// Signature of unencrypted seed entropy
        )? != seed_entropy {
            return Err(ConfigurationError::Generic(format!(
                "Unable to recover seed entropy of config: {:?}", config
            )))
        };
        Ok(config)
    }
}

/// encrypt_seed_with -- sign with admin_and encrypt the seed with the given seed "blinding" key
pub fn encrypt_seed_with(
    seed: &[u8],
    seed_key: &[u8]
) -> Result<Vec<u8>, ConfigurationError> {
    let mut seed_enc = aes::ecb_encryptor(
        aes::KeySize::KeySize256,
        &seed_key,
        blockmodes::NoPadding
    );
    let mut seed_cipher = Vec::<u8>::new();
    let mut reader = RefReadBuffer::new(seed);
    let mut buffer = [0; 4096];
    let mut writer = RefWriteBuffer::new(&mut buffer);
    // See: https://github.com/buttercup/rust-crypto-wasm/blob/master/examples/symmetriccipher.rs
    loop {
        let result = seed_enc.encrypt(&mut reader, &mut writer, true)?;
        seed_cipher.extend(writer.take_read_buffer().take_remaining().iter().map(|&i| i));
        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => { },
        }
    };
    Ok(seed_cipher)
}

pub fn decrypt_seed_with(
    admin_pubkey:	&str,   // HcA...
    seed_key:		&str,   // HcB...
    seed_cipher:	&str,	// HcC...
    seed_sig:		&str,	// Base-64
) -> Result<Vec<u8>, ConfigurationError> {
    let seed_key_dec = hcid::HcidEncoding::with_kind("hcb0")?.decode(seed_key)?;
    let mut seed_dec = aes::ecb_decryptor(
        aes::KeySize::KeySize256,
        &seed_key_dec,
        blockmodes::NoPadding
    );
    let mut seed = Vec::<u8>::new();
    let seed_cipher_data = hcid::HcidEncoding::with_kind("hcc0")?.decode(seed_cipher)?;
    let mut reader = RefReadBuffer::new(&seed_cipher_data);
    let mut buffer = [0; 4096];
    let mut writer = RefWriteBuffer::new(&mut buffer);
    loop {
        let result = seed_dec.decrypt(&mut reader, &mut writer, true)?;
        seed.extend(writer.take_read_buffer().take_remaining().iter().map(|&i| i));
        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => { },
        }
    };

    // Finally, check the signature on the seed against the admin_pubkey
    let admin_pubkey_data = hcid::HcidEncoding::with_kind("hca0")?.decode(admin_pubkey)?;
    let pubkey = PublicKey::from_bytes(&admin_pubkey_data)?;
    let sig = Signature::from_bytes(&base64::decode(seed_sig)?)?;
    pubkey.verify(&seed, &sig)?;

    Ok(seed)
}
        
/// admin_key_from -- Stretches the email (salt) + password + name, generates "admin" keypair
/// blind_key_from -- Uses a provided private key to create a new sub-key
/// 
/// TODO: We must be able to generate a sequence of unique admin keypairs for each unique HoloPort.
/// This is also required for (later) when we support DPKI-generated entropy/keypairs.
pub fn admin_key_from(
    email:		&str,
    password:		&str,
    name_maybe:		&Option<String>,
) -> Result<Keypair, ConfigurationError> {
    // Extend the email address to a 256-bit salt using SHA-256.  This prevents very short
    // email addresses (eg. a@b.ca) from triggering salt size related failures in argon2.
    let mut hasher = sha2::Sha256::new();
    hasher.input_str(email);
    let mut salt = [0u8; 32];
    hasher.result(&mut salt);

    // Extend the password, including the (optional) name.
    hasher.reset();
    hasher.input_str(password);
    if let Some(name) = name_maybe {
        hasher.input_str(name)
    }
    let mut pass = [0u8; 32];
    hasher.result(&mut pass);

    // Extend the hashed email salt + password (+ nonce) into a seed for the admin Keypair
    keypair_from_seed(
        &argon2::hash_raw(
            &pass, &salt, &HOLO_ADMIN_ARGON_CONFIG
        )?
    )
}
            
pub fn blind_key_from(
    admin_key: &[u8; 32]
) -> Result<Keypair, ConfigurationError> {
    keypair_from_seed(admin_key)
}

pub fn keypair_from_seed(seed: &[u8]) -> Result<Keypair, ConfigurationError> {
    let secret: SecretKey = SecretKey::from_bytes(seed)?;
    let public: PublicKey = (&secret).into();
    Ok(Keypair{ public, secret })
}


    
/// 
/// authenticate_monotonically -- Tests the payload w/ supplied timestamp % duration
/// 
/// The JSON tested is simply: "[<prefix>, <payload>]", with the prefix allowing the current and
/// immediately previous time periods around `instant`, as defined by `duration`.
///
/// ```
/// let keypair = holo_configure::keypair_from_seed( &[0u8; 32] ).unwrap();
/// let now = 35*365*24*60*60; // 35 years past the UNIX Epoch
/// let delta = 1*60*60;       // allow +/- 1 hr.
/// let prefix = ( now+5 ) / delta;
/// let sig = holo_configure::sign_with_prefix(
///         "\"Hello, World\"",
///         prefix,
///         &keypair
/// );
/// assert_eq!(
///     holo_configure::authenticate_monotonically(
///         "\"Hello, World\"",
///         std::time::UNIX_EPOCH + std::time::Duration::from_secs( now+10 ),
///         std::time::Duration::from_secs( delta ),
///         &keypair,
///         &sig
///     ).unwrap(),
///     true
/// );
/// assert_eq!(
///     holo_configure::authenticate_monotonically(
///         "\"Hello, World\"",
///         std::time::UNIX_EPOCH + std::time::Duration::from_secs( now-10 ),
///         std::time::Duration::from_secs( delta ),
///         &keypair,
///         &sig
///     ).unwrap(),
///     false
/// );
/// assert_eq!(
///     holo_configure::authenticate_monotonically(
///         "\"Hello, World\"",
///         std::time::UNIX_EPOCH + std::time::Duration::from_secs( now+delta ),
///         std::time::Duration::from_secs( delta ),
///         &keypair,
///         &sig
///     ).unwrap(),
///     true
/// );
/// assert_eq!(
///     holo_configure::authenticate_monotonically(
///         "\"Hello, World\"",
///         std::time::UNIX_EPOCH + std::time::Duration::from_secs( now+delta*2 ),
///         std::time::Duration::from_secs( delta ),
///         &keypair,
///         &sig
///     ).unwrap(),
///     false
/// );
/// ```
/// 
pub fn sign_with_prefix(
    payload:		&str, // JSON
    prefix:		u64,
    keypair: 		&Keypair
) -> String {
    eprintln!("Prefix: {}", prefix);
    base64::encode(
        &keypair.sign(
            &format!("[{}, {}]", prefix, payload).as_bytes()
        ).to_bytes().into_iter()
    )
}

pub fn authenticate_monotonically(
    payload:		&str, // JSON
    timestamp: 		SystemTime,
    delta: 		Duration,
    keypair: 		&Keypair,
    sig: 		&str
) -> Result<bool, ConfigurationError> {
    let seconds: u64 = timestamp.duration_since(SystemTime::UNIX_EPOCH)?.as_secs();
    let prefix: u64 = seconds / delta.as_secs();

    Ok(sig == sign_with_prefix(payload, prefix - 0, keypair)
       || ( prefix > 0
            && sig == sign_with_prefix(payload, prefix - 1, keypair)))
}

/// Create a unique HoloPort configuration, w/ random seed entropy
///
/// Lets create a HoloPortConfiguration with a deterministic (all zeros) seed entropy:
/// ```
/// let config = holo_configure::holoport_configuration(
///     Some("HP1".to_string()), "a@b.c".to_string(), "password".to_string(), Some([0u8; 32])
/// );
/// assert_eq!(serde_json::to_string_pretty( &config.unwrap() ).unwrap(),
/// "{
///   \"name\": \"HP1\",
///   \"email\": \"a@b.c\",
///   \"admin_pubkey\": \"HcAcIwy3I4KPhtwqhnBtPRMFhqzyasf8yW6SMeoQF5Hwxnhsafg5Qn33qyb7eda\",
///   \"seed_key\": \"HcBCjuRDXiQy7oivv78z3Ozjq3YW97mpcj38UQcVQ4PYbxbtd84XVWjebm7vvwi\",
///   \"seed\": \"HcCCJ6jX98BJRIrhba9T4s9WYIu5S3Qsg59ZfgBCA6ed8mkh8X7CqpHfGZmxv8a\",
///   \"seed_sig\": \"Yt1EDgH39lbe4SLz1C6N6SS/b3o7qOgKAPKLbsZB3Q1ODqRr/OgSxnMMLlrJPEX4j5epZ2aSaWVEL7NQ76vaBg==\"
/// }"
/// );
/// ```
/// 
pub fn holoport_configuration(
    name_maybe:		Option<String>,
    email:		String,
    password:		String,
    seed_maybe:		Option<[u8; HOLO_ENTROPY_SIZE]>
) -> Result<HoloPortConfiguration, ConfigurationError> {
    Ok(HoloPortConfiguration::new(name_maybe, email, password, seed_maybe)?)
}
