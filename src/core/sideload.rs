use isideload::{AppleId, SideloadSession, App};
use anyhow::{Result, Context};
use std::path::PathBuf;

pub async fn resign_and_install(
    apple_id: &str,
    password: &str,
    ipa_path: &str,
    udid: &str,
) -> Result<u8> {
    // 1. Initialize isideload (Required to catch network/crypto errors properly)
    isideload::init();

    // 2. Authenticate with Apple Developer Servers
    let account = AppleId::new(apple_id.to_string(), password.to_string());
    let session = SideloadSession::login(account).await
        .context("Failed to log in. Check your Apple ID or App-Specific Password.")?;

    // 3. Register Device UDID to the Apple ID Developer Team
    session.register_device(udid).await
        .context("Failed to register device with Apple Developer portal.")?;

    // 4. Parse the IPA, inject the Free Developer Provisioning Profile, and Sign
    let app = App::from_ipa(ipa_path)
        .context("Failed to parse IPA file.")?;
    
    let signed_ipa_path: PathBuf = session.sign_app(&app).await
        .context("Code signing failed. Check apple-codesign logs.")?;

    // 5. Hand over to idevice for installation
    super::device::install_ipa(udid, signed_ipa_path.to_str().unwrap())?;

    // Returns days remaining for free developer certificates
    Ok(7) 
}