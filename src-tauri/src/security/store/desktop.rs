// 데스크탑(개발·테스트) 보안저장: OS 키체인.
// macOS Keychain / Windows Credential Manager / Linux kernel keyutils.
use crate::security::SERVICE;
use anyhow::Result;

fn entry(account: &str) -> Result<keyring::Entry> {
    Ok(keyring::Entry::new(SERVICE, account)?)
}

pub fn set(account: &str, secret: &str) -> Result<()> {
    entry(account)?.set_password(secret)?;
    Ok(())
}

pub fn get(account: &str) -> Result<String> {
    Ok(entry(account)?.get_password()?)
}

pub fn exists(account: &str) -> Result<bool> {
    match entry(account)?.get_password() {
        Ok(_) => Ok(true),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(e) => Err(e.into()),
    }
}

pub fn delete(account: &str) -> Result<()> {
    match entry(account)?.delete_credential() {
        Ok(()) => Ok(()),
        // 이미 없으면 삭제 성공으로 취급 (멱등).
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
