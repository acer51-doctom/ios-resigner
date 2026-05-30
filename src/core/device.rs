use idevice::{Device, LockdowndClient};
use anyhow::{Result, Context};

pub struct ConnectedDevice {
    pub udid: String,
    pub name: String,
}

/// Finds the first connected iOS device and ensures it is paired.
pub fn get_and_pair_device() -> Result<ConnectedDevice> {
    // Fetch devices via usbmuxd
    let devices = Device::get_devices()
        .context("Failed to connect to usbmuxd. Is iTunes/usbmuxd installed?")?;
    
    let device = devices.into_iter().next()
        .context("No iOS device connected. Plug it in via USB.")?;

    let udid = device.get_udid();
    
    // Connect to lockdown daemon to pair
    let lockdown = LockdowndClient::new(&device, "ios-resigner")
        .context("Could not connect to lockdown daemon")?;

    let device_name = lockdown.get_device_name().unwrap_or_else(|_| "Unknown iPhone".into());

    // iLoader Pairing Logic snippet
    if lockdown.validate_pair().is_err() {
        println!("Device not trusted. Please tap 'Trust' on your iPhone.");
        
        // Loop until the user taps Trust
        loop {
            match lockdown.pair() {
                Ok(_) => break,
                Err(_) => {
                    std::thread::sleep(std::time::Duration::from_secs(2));
                }
            }
        }
    }

    Ok(ConnectedDevice { udid, name: device_name })
}

/// Installs the signed IPA to the device via installd
pub fn install_ipa(udid: &str, ipa_path: &str) -> Result<()> {
    let device = Device::new(udid).context("Device lost during installation")?;
    let installd = device.new_installd_client("ios-resigner")
        .context("Failed to start installd")?;

    installd.install(ipa_path, true)
        .context("Failed to install IPA to device")?;

    Ok(())
}