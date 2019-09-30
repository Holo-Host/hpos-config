use crate::Seed;

use holochain_conductor_api::key_loaders::mock_passphrase_manager;
use holochain_conductor_api::keystore::*;
use holochain_common::DEFAULT_PASSPHRASE;
use holochain_dpki::CODEC_HCS0;

use ed25519_dalek::PublicKey;
use failure::Error;
use url::Url;

pub fn public_key_as_url(public_key: PublicKey) -> Result<Url, Error> {
    let url = Url::parse(&format!("https://{}.holohost.net", CODEC_HCS0.encode(&public_key.to_bytes())?))?;
    Ok(url)
}

pub fn from_seed(seed: &Seed) -> Result<(Keystore, PublicKey), Error> {
    let passphrase_manager = mock_passphrase_manager(DEFAULT_PASSPHRASE.into());
    let mut keystore = Keystore::new(passphrase_manager, None)?;

    keystore.add_seed(STANDALONE_ROOT_SEED, seed)?;
    let (public_key_hcid, _) = keystore.add_keybundle_from_seed(STANDALONE_ROOT_SEED, PRIMARY_KEYBUNDLE_ID)?;
    let public_key_bytes = CODEC_HCS0.decode(&public_key_hcid)?;
    let public_key = PublicKey::from_bytes(&public_key_bytes)?;

    Ok((keystore, public_key))
}
