# Index for hpos-config

###### tags: `Holo` `documentation`

This is meant to track the use of [Config in other repos](https://github.com/Holo-Host/hpos-config/blob/develop/core/src/config.rs#L59)

We will be pointing out all the services in holo-nixpkgs tha requires to be changed.

### Dependencies for hpos-config:

- [hpos-auth](https://github.com/Holo-Host/holo-auth)
  - update client
- [hpos-configure-holochain](https://github.com/Holo-Host/hpos-configure-holochain):
  - Used to install the host agent key from the config.
- [router-registry (holo-router)](https://github.com/Holo-Host/router-registry)
  - update `holo-router-agent` crate
- [@holo-nicpkgs/hpos-admin-api]():
  - api should be able to read the right struct of the config to return the appropriate values back
- [@holo-nicpkgs/hpos-admin-client]():
  - cli docs need to be updated
- [hp-admin-crypto]()
  - ~~

### Note some features to test:

- Host uses hpos-config keys as holochain keys in `alphaNet/mainNet`.
  - note this would involve seeing that lair is initialized with the right keys too.
- hpos-admin-api endpoints like `get-settings`
