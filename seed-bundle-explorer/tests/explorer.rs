#[cfg(test)]
mod tests {
    use ed25519_dalek::VerifyingKey;
    use hpos_config_core::Config;
    use hpos_config_seed_bundle_explorer::holoport_public_key;

    #[tokio::test(flavor = "multi_thread")]
    async fn get_sign_pub_key() -> Result<(), String> {
        let config: Config = get_mock_config()?;
        let pub_key = holoport_public_key(&config, Some("pass".to_string()))
            .await
            .unwrap();
        // TODO: Update the bundle that was generate from the gen-web
        assert_eq!(pub_key, get_mock_pub_key()?);
        Ok(())
    }
    fn get_mock_config() -> Result<Config, String> {
        let email: String = "jack@holo.host".to_string();
        let password: String = "password".to_string();
        let registration_code: String = "registration-code".to_string();
        let derivation_path: String = "1".to_string();
        let device_bundle = "k6VoY3NiMJGWonB3xBCZ0R47aR6ctMScaYsrOLwRzSAAAcQY58NsOmNCDbniGsLgUhj5UoHjBrapiiDGxDGAa5Wqzm0pVuXGN106iyMHRk4dOf0iGWj65oCeB8-ZYXJdeflsVDY-DOuJaadfPZQExCyCrWRldmljZV9udW1iZXIAq2dlbmVyYXRlX2J5r3F1aWNrc3RhcnQtdjIuMA".to_string();
        let rev_key: [u8; 32] = [0 as u8; 32]; // TODO: Fill this in with something
        let (config, _) = Config::new(
            email,
            password,
            registration_code,
            VerifyingKey::from_bytes(&rev_key).unwrap(),
            derivation_path,
            device_bundle,
            get_mock_pub_key()?,
        )
        .unwrap();
        Ok(config)
    }
    fn get_mock_pub_key() -> Result<VerifyingKey, String> {
        let device_pub_key: String = "To4PzBU8BcVghpjGjnYjLQnP_mkT9uBJ2v969Cs7-xw".to_string();
        Ok(
            base64::decode_config(&device_pub_key, base64::URL_SAFE_NO_PAD)
                .map(|bytes| VerifyingKey::from_bytes(&bytes[0..32].try_into().unwrap()))
                .unwrap()
                .unwrap(),
        )
    }
}
