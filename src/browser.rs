use anyhow::{anyhow, Result};
use std::process::Command;

/// Extract the `sessionKey` cookie for claude.ai from Chrome's local cookie database.
/// macOS only — requires Chrome and Keychain access.
pub fn extract_chrome_cookie() -> Result<String> {
    // 1. Get Chrome Safe Storage password from macOS Keychain
    let output = Command::new("security")
        .args(["find-generic-password", "-w", "-s", "Chrome Safe Storage"])
        .output()?;
    if !output.status.success() {
        return Err(anyhow!(
            "Failed to get Chrome Safe Storage password from Keychain.\n\
             Make sure Chrome is installed and you've granted Keychain access."
        ));
    }
    let password = String::from_utf8(output.stdout)?.trim().to_string();

    // 2. Copy Chrome's Cookies DB to temp (Chrome locks the file)
    let home = dirs::home_dir().ok_or_else(|| anyhow!("No home directory"))?;
    let cookies_db = home.join("Library/Application Support/Google/Chrome/Default/Cookies");
    if !cookies_db.exists() {
        return Err(anyhow!(
            "Chrome cookies database not found at {:?}\n\
             Make sure you're logged into claude.ai in Chrome.",
            cookies_db
        ));
    }
    let tmp_db = std::env::temp_dir().join("claude-tui-chrome-cookies.db");
    std::fs::copy(&cookies_db, &tmp_db)?;

    // 3. Query for sessionKey using sqlite3 CLI (pre-installed on macOS)
    let query = "SELECT hex(encrypted_value) FROM cookies \
                 WHERE host_key LIKE '%claude.ai%' AND name = 'sessionKey' \
                 ORDER BY last_access_utc DESC LIMIT 1;";
    let output = Command::new("sqlite3")
        .arg(&tmp_db)
        .arg(query)
        .output()?;
    let _ = std::fs::remove_file(&tmp_db);

    if !output.status.success() {
        return Err(anyhow!("sqlite3 query failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    let hex_encrypted = String::from_utf8(output.stdout)?.trim().to_string();
    if hex_encrypted.is_empty() {
        return Err(anyhow!(
            "No sessionKey cookie found for claude.ai in Chrome.\n\
             Make sure you're logged into https://claude.ai in Chrome."
        ));
    }

    // 4. Decrypt the cookie value
    let encrypted = hex::decode(&hex_encrypted)?;

    // Chrome v10 format: "v10" (3 bytes) + IV (16 bytes) + AES-128-CBC ciphertext
    if encrypted.len() < 20 || &encrypted[..3] != b"v10" {
        return Err(anyhow!("Unexpected cookie encryption format (expected v10 prefix)"));
    }
    let iv: [u8; 16] = encrypted[3..19].try_into().unwrap();
    let encrypted_data = &encrypted[19..];

    if encrypted_data.len() % 16 != 0 {
        return Err(anyhow!(
            "Encrypted data length ({}) is not a multiple of AES block size",
            encrypted_data.len()
        ));
    }

    // Derive key: PBKDF2-HMAC-SHA1(password, "saltysalt", 1003 iterations, 16 bytes)
    let mut key = [0u8; 16];
    pbkdf2::pbkdf2_hmac::<sha1::Sha1>(password.as_bytes(), b"saltysalt", 1003, &mut key);

    // AES-128-CBC decrypt with embedded IV
    use aes::cipher::{BlockDecryptMut, KeyIvInit};
    type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

    let mut buf = encrypted_data.to_vec();
    let decrypted = Aes128CbcDec::new(&key.into(), &iv.into())
        .decrypt_padded_mut::<aes::cipher::block_padding::Pkcs7>(&mut buf)
        .map_err(|e| anyhow!("AES decryption failed: {}", e))?;

    // Decrypted data has a 16-byte prefix before the actual cookie value
    if decrypted.len() <= 16 {
        return Err(anyhow!("Decrypted data too short"));
    }
    let cookie_value = String::from_utf8(decrypted[16..].to_vec())?;

    if !cookie_value.starts_with("sk-ant-") {
        return Err(anyhow!("Decrypted value doesn't look like a session key"));
    }

    Ok(cookie_value)
}
