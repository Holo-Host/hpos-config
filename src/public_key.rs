use ed25519_dalek::PublicKey;
use failure::Error;
use hcid::HcidEncoding;
use lazy_static::lazy_static;
use url::Url;

lazy_static! {
    pub static ref HCID_CODEC: hcid::HcidEncoding =
        HcidEncoding::with_kind("hcs0").expect("Couldn't init hcs0 hcid codec.");
}

pub fn to_hcid(public_key: &PublicKey) -> Result<String, Error> {
    let hcid = HCID_CODEC.encode(&public_key.to_bytes())?;
    Ok(hcid)
}

pub fn to_url(public_key: &PublicKey) -> Result<Url, Error> {
    let hcid = to_hcid(&public_key)?;
    let url = Url::parse(&format!("https://{}.holohost.net", hcid))?;
    Ok(url)
}
