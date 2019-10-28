pub mod config;
pub mod public_key;


pub use config::{Config, keypair_from, admin_keypair_from};

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[test]
    fn hc_public_key_url() {
        let email: String = "pj@aa.pl".to_string();
        let password: String =  "password".to_string();
        let seed: Option<[u8; 32]> = Some([55; 32]);
        let expected_url = "https://hcscimeesmngnuygkhtit5auwbfiuivxjmff7o54speb6zg84yebxuv7bf7z58z.holohost.net/";

        let (_, public_key) = Config::new(email, password, seed).unwrap();
        assert_eq!(public_key::to_url(&public_key).unwrap(), Url::parse(expected_url).unwrap());
    }

    #[test]
    fn hc_public_key_hcid() {
        let email: String = "pj@aa.pl".to_string();
        let password: String =  "password".to_string();
        let seed: [u8; 32] = [55; 32];
        let expected_hcid: String = "HcScIMeeSmNgnuygkhTIT5auWbfiuivxjMfF7O54sPeb6zg84yEBXUV7bf7z58z".to_string();

        let (_, public_key) = Config::new(email, password, Some(seed)).unwrap();
        assert_eq!(public_key::to_hcid(&public_key).unwrap(), expected_hcid);
    }
}
