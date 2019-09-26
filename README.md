# Configure a Holo HoloPortOS Instance

Deploying a basic configuration to a HoloPortOS instance requires key generation material and basic
identity + password to be collected and deployed.

## USB Configuration

The simplest and most direct method is to generate a configuration, and copy it onto a USB stick,
which is then inserted into the HoloPortOS instance.  When the device boots, it will:

- Use the data on the USB stick to create its Holochain and ZeroTier keys
- Authenticate itself to the Holo ZeroTier network, which will provision its DNS configuration
- Start the Holo services
- Eject the USB and blacklist the kernel modules

## Building

```
$ nix-build -A holo-configure
```

If that doesn't work:
```
$ nix-shell
$ cargo build --lib --bin holo-configure
```

## Generating a `HoloPortConfiguration`

```
$ ./target/debug/holo-configure --name "HP1" --email "a@b.ca" --password "secret" | tee holo.json
Generating HoloPort Configuration for email: a@b.ca
{
  "name": "HP1",
  "email": "a@b.ca",
  "admin_pubkey": "HcACj5GG78Nfa476fdvcAXwOQdv7hq7gyHi6bueg7ZSb4iix5hNpUcDFjnjejvi",
  "seed_key": "HcBciRfTSA9E9h6y9mdfzi95PPYhkj6qfBHsXRAC34hpqjx3kxHNIWKYzexuzva",
  "seed": "HcCcjf5ebsEDxmiz8j7YZX3r47qriFhwwPFZc7Vc7Wys5ozcFwUAQQTXoaygnbr",
  "seed_sig": "WO0PYkFg1RZEP1UOzdBacj5QtHuM37uqjn0zPSSsgw8gJX2TU4NoQNb3tDMNvSFK5n4dDcen10ScGsRIde5iCA=="
}
```

### `name`

Optionally, make the `admin_key` and `seed_key` unique, by hashing the supplied `name` into
`password` when performing the Argon2 password hashing.

### `email` and `password`

These will be harvested from stdin, if not supplied on the command-line.

The `email` is used as a salt (SHA-2 256 hashed, to avoid too-short email addresses).

The `password` is hashed (optionally, with any supplied `name`), and the resultant salt and password
hashes are used to generate an argon2 password hash, which is used to generate the Admin and
Blinding keypairs.

### `admin_pubkey: String` "HcAcj..."

The Admin Private key is used to sign all HoloPort admin requests; the `admin_pubkey` is used to
authenticate them.

### `seed_key: Option<String>` "HcBci..."

A 256-bit AES ECB encryption key is *always* used to encrypt the seed; it is *optionally* supplied
here in the config.

### `seed: String` "HcCcf..."

The encrypted seed entropy; decrypted using the `seed_key` (if supplied).

#### Future: Admin Decryption at Holo Boot

If not supplied, then the seed material must be decrypted by a holder of the admin private key (who
can generate the `seed_key`, and decrypt the `seed`).  This would require the Admin password-holder
to log into the HoloPort and decrypt the `seed`, before Holo could use the decrypted seed entropy to
generate the Holo Agent and ZeroTier Keypairs, and continue to boot up.

## `holo-configure` APIs

### `holoport_configuration`

Returns the `HoloPortConfiguration` object, which can be serialized to JSON.

#### `name_maybe: &Option<String>`
#### `email:      &str,
#### `password:   &str,
#### `seed_maybe: Option<[u8; HOLO_ENTROPY_SIZE]>

