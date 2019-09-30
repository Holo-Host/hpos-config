use holochain_dpki::CODEC_HCS0;

use ed25519_dalek::PublicKey;
use failure::Error;
use url::Url;

pub fn to_hcid(public_key: &PublicKey) -> Result<String, Error> {
    let hcid = CODEC_HCS0.encode(&public_key.to_bytes())?;
    Ok(hcid)
}

pub fn to_url(public_key: &PublicKey) -> Result<Url, Error> {
    let hcid = to_hcid(&public_key)?;
    let url = Url::parse(&format!("https://{}.holohost.net", hcid))?;
    Ok(url)
}
