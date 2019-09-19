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
