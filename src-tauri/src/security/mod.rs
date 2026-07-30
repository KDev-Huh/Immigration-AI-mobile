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
    Gemini,
}

impl CloudProvider {
    /// 보안저장 계정 키.
    pub fn account(self) -> &'static str {
        match self {
            CloudProvider::Openai => "openai_api_key",
            CloudProvider::Anthropic => "anthropic_api_key",
            CloudProvider::Gemini => "gemini_api_key",
        }
    }
}

/// 저장 전 정규화·검증.
///
/// API 키에는 공백이 들어갈 수 없다. 그런데 모바일에서 키를 붙여넣으면 줄바꿈이나
/// 공백이 **중간에** 섞여 들어오는 일이 흔하다(메일·메모 앱에서 복사할 때 줄이 접힘).
/// 양끝만 트림하면 손상된 키가 조용히 저장되고, 나중에 API 가 401 을 주는데
/// 사용자 입장에서는 "맞는 키인데 왜 안 되지"로만 보인다.
/// → 모든 공백을 제거해 그 실패 유형을 원천 차단한다.
fn validate(key: &str) -> Result<String> {
    let k: String = key.chars().filter(|c| !c.is_whitespace()).collect();
    if k.is_empty() {
        return Err(anyhow!("API 키가 비어 있습니다"));
    }
    // 제어문자가 남아 있으면 붙여넣기가 깨진 것 — 저장해도 반드시 실패한다.
    if k.chars().any(|c| c.is_control()) {
        return Err(anyhow!(
            "API 키에 이상한 문자가 섞여 있습니다. 다시 복사해서 붙여넣어 주세요."
        ));
    }
    Ok(k)
}

pub fn set_api_key(provider: CloudProvider, key: &str) -> Result<()> {
    let key = validate(key)?;
    store::set(provider.account(), &key)
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
        assert_ne!(
            CloudProvider::Openai.account(),
            CloudProvider::Gemini.account()
        );
        assert_ne!(
            CloudProvider::Anthropic.account(),
            CloudProvider::Gemini.account()
        );
    }

    #[test]
    fn blank_key_rejected() {
        assert!(validate("   ").is_err());
        assert!(validate("").is_err());
        assert!(validate("\n\t").is_err());
        assert_eq!(validate("  sk-abc  ").unwrap(), "sk-abc"); // 양끝 공백 절삭
    }

    #[test]
    fn internal_whitespace_stripped() {
        // 모바일에서 붙여넣을 때 줄이 접혀 개행·공백이 섞이는 사고가 흔하다.
        // 이게 남으면 401 이 나는데 원인을 알 수 없다.
        assert_eq!(validate("sk-proj-abc\ndef").unwrap(), "sk-proj-abcdef");
        assert_eq!(
            validate("sk-proj-abc def\r\nghi").unwrap(),
            "sk-proj-abcdefghi"
        );
    }

    #[test]
    fn long_project_key_survives_intact() {
        // sk-proj- 키는 160자대다. 정규화가 길이를 건드리면 안 된다.
        let key = format!("sk-proj-{}ZZZZ", "A".repeat(152));
        assert_eq!(key.len(), 164);
        assert_eq!(validate(&key).unwrap(), key);
    }

    #[test]
    fn control_chars_rejected() {
        assert!(validate("sk-proj-abc\u{0}def").is_err());
    }

    #[test]
    fn provider_serde_is_lowercase() {
        // 프론트 CloudProvider 타입("openai"|"anthropic"|"gemini")과의 계약.
        let j = serde_json::to_string(&CloudProvider::Openai).unwrap();
        assert_eq!(j, "\"openai\"");
        let p: CloudProvider = serde_json::from_str("\"anthropic\"").unwrap();
        assert_eq!(p, CloudProvider::Anthropic);
        let g: CloudProvider = serde_json::from_str("\"gemini\"").unwrap();
        assert_eq!(g, CloudProvider::Gemini);
    }
}
