// 자격증명 보관. **평문 파일 저장 절대 금지** — OS 보안저장만 사용한다.
//   iOS     : Keychain Services
//   Android : AndroidKeyStore(AES-GCM) 로 암호화 후 앱 전용 SharedPreferences 에 보관
//   데스크탑: OS 키체인 (keyring crate) — 개발·테스트용
//
// 백엔드 확보에 실패하면 **저장하지 않고 에러**를 반환한다. 폴백으로 평문에 쓰는 경로는 없다.
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

mod store;

/// 키체인 서비스 식별자 — 앱 번들 ID 와 일치.
pub const SERVICE: &str = "com.immigrationai.mobile";

/// 클라우드 공급자. 브라우저 로그인은 미지원(임베딩 API 부재 + 약관 위험).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum CloudProvider {
    Openai,
    Anthropic,
}

impl CloudProvider {
    /// 보안저장 계정 키.
    pub fn account(self) -> &'static str {
        match self {
            CloudProvider::Openai => "openai_api_key",
            CloudProvider::Anthropic => "anthropic_api_key",
        }
    }
}

/// 저장 전 최소 검증. 공백 키를 저장해 "키 있음"으로 오인되는 것을 막는다.
fn validate(key: &str) -> Result<&str> {
    let k = key.trim();
    if k.is_empty() {
        return Err(anyhow!("API 키가 비어 있습니다"));
    }
    Ok(k)
}

pub fn set_api_key(provider: CloudProvider, key: &str) -> Result<()> {
    let key = validate(key)?;
    store::set(provider.account(), key)
}

pub fn get_api_key(provider: CloudProvider) -> Result<String> {
    store::get(provider.account())
}

pub fn has_api_key(provider: CloudProvider) -> Result<bool> {
    store::exists(provider.account())
}

pub fn delete_api_key(provider: CloudProvider) -> Result<()> {
    store::delete(provider.account())
}

/// 임베딩 전용 키 조회. 채팅 공급자와 무관하게 **항상 OpenAI** 키가 필요하다.
pub fn embedding_api_key() -> Result<String> {
    get_api_key(CloudProvider::Openai).map_err(|_| {
        anyhow!("OpenAI API 키가 없습니다. 임베딩에는 OpenAI 키가 반드시 필요합니다 (설정 탭에서 등록).")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accounts_are_distinct() {
        assert_ne!(
            CloudProvider::Openai.account(),
            CloudProvider::Anthropic.account()
        );
    }

    #[test]
    fn blank_key_rejected() {
        assert!(validate("   ").is_err());
        assert!(validate("").is_err());
        assert_eq!(validate("  sk-abc  ").unwrap(), "sk-abc"); // 공백 절삭
    }

    #[test]
    fn provider_serde_is_lowercase() {
        // 프론트 CloudProvider 타입("openai"|"anthropic")과의 계약.
        let j = serde_json::to_string(&CloudProvider::Openai).unwrap();
        assert_eq!(j, "\"openai\"");
        let p: CloudProvider = serde_json::from_str("\"anthropic\"").unwrap();
        assert_eq!(p, CloudProvider::Anthropic);
    }
}
