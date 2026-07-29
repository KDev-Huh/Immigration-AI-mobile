// 문서 도메인: 메타·태그·저장(store) + 파싱 + 청킹.
pub mod chunker;
pub mod parser;
pub mod store;

use serde::{Deserialize, Serialize};

/// 유출 분류 태그.
///
/// 모바일은 클라우드 전송(임베딩·생성)이 전제이므로 `Confidential` 문서는
/// **업로드 자체가 금지**된다. 값을 남겨둔 이유는 거부 판정을 명시적으로
/// 표현하기 위함이며, 저장소에 Confidential 레코드가 생기는 일은 없다.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Sensitivity {
    Leakable,
    Confidential,
}

impl Sensitivity {
    /// 모바일 업로드 허용 여부. 최후 방어선 — 프론트 검증과 무관하게 여기서 막는다.
    pub fn allows_upload(self) -> bool {
        matches!(self, Sensitivity::Leakable)
    }
}

/// 업로드 거부 사유 (프론트에 그대로 노출).
pub const UPLOAD_REJECT: &str =
    "유출불가 문서는 모바일에서 업로드할 수 없습니다 (클라우드 전송 전제). 데스크탑판을 사용하세요.";

/// 프론트에 노출되는 문서 메타 (types.ts DocumentMeta 와 계약).
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocumentMeta {
    pub id: String,
    pub filename: String,
    pub sensitivity: Sensitivity,
    pub pages: u32,
    pub chunk_count: u32,
    pub status: String, // pending|parsing|chunking|embedding|ready|error
    pub progress: f32,  // 0..1
    pub updated_at: String,
    pub error: Option<String>,
}

/// 청크 1개 — 임베딩·검색·출처의 최소 단위.
#[derive(Serialize, Deserialize, Clone)]
pub struct Chunk {
    pub doc_id: String,
    pub index: usize,
    pub text: String,
    pub page: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confidential_upload_is_rejected() {
        assert!(!Sensitivity::Confidential.allows_upload());
        assert!(Sensitivity::Leakable.allows_upload());
    }
}
