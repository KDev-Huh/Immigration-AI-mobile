// iOS 보안저장: Keychain Services (generic password).
// 키는 앱 샌드박스 키체인에만 남고 백업/파일시스템에 평문으로 노출되지 않는다.
use crate::security::SERVICE;
use anyhow::{anyhow, Result};
use security_framework::passwords::{
    delete_generic_password, get_generic_password, set_generic_password,
};

pub fn set(account: &str, secret: &str) -> Result<()> {
    set_generic_password(SERVICE, account, secret.as_bytes())
        .map_err(|e| anyhow!("Keychain 저장 실패: {e}"))
}

pub fn get(account: &str) -> Result<String> {
    let bytes =
        get_generic_password(SERVICE, account).map_err(|e| anyhow!("Keychain 조회 실패: {e}"))?;
    String::from_utf8(bytes).map_err(|_| anyhow!("Keychain 값이 UTF-8 이 아닙니다"))
}

pub fn exists(account: &str) -> Result<bool> {
    // 조회 실패는 "없음"으로 간주 — 존재 여부 확인이 목적이라 오류 구분이 불필요.
    Ok(get_generic_password(SERVICE, account).is_ok())
}

pub fn delete(account: &str) -> Result<()> {
    // 이미 없으면 성공으로 취급 (멱등).
    let _ = delete_generic_password(SERVICE, account);
    Ok(())
}
