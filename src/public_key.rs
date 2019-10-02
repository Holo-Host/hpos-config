use ed25519_dalek::PublicKey;
use failure::Error;
use url::Url;

use holochain_dpki::CODEC_HCS0;

pub fn to_hcid(public_key: &PublicKey) -> Result<String, Error> {
    let hcid = CODEC_HCS0.encode(&public_key.to_bytes())?;
    Ok(hcid)
}

pub fn to_url(hcid: &str) -> Result<Url, Error> {
    let url = Url::parse(&format!("https://{}.holohost.net", hcid))?;
    Ok(url)
}
