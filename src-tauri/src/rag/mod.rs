// RAG 오케스트레이션 상수·계약 타입 + 순수 로직.
// 실제 상태(store) 접근·네트워크 호출은 commands 에서.
pub mod embedding;
pub mod retriever;
pub mod vectordb;

use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Citation {
    pub doc_id: String,
    pub filename: String,
    pub page: u32,
    pub snippet: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct ChatAnswer {
    pub text: String,
    pub citations: Vec<Citation>,
}

/// 검색 top-k.
pub const TOP_K: usize = 12;
/// 컨텍스트 문자 예산.
pub const CONTEXT_BUDGET_CHARS: usize = 8000;
/// 하이브리드 검색 어휘 가중치 (0=벡터만, 1=어휘만).
pub const HYBRID_LAMBDA: f32 = 0.4;
/// 최고 점수가 이 미만이면 근거 부족 → "자료 없음".
pub const RELEVANCE_MIN: f32 = 0.2;
/// 근거 부족 시 고정 응답.
pub const NO_EVIDENCE: &str = "자료 없음 — 업로드된 문서에서 근거를 찾지 못했습니다.";

pub const SYSTEM_PROMPT: &str = "당신은 비자·체류 상담을 돕는 행정 보조 AI입니다. \
반드시 아래 제공된 문서 발췌(컨텍스트)에 근거해서만 답하세요. \
컨텍스트에 없는 내용은 지어내지 말고 모른다고 하세요. \
답변 끝에 근거가 된 파일명과 페이지를 밝히세요.";

/// 클라우드 API 의 user 메시지 본문 조립 (system 은 별도 필드로 전달).
pub fn build_user_message(context: &str, query: &str) -> String {
    format!("=== 컨텍스트 ===\n{context}\n=== 질문 ===\n{query}")
}

/// 근거 충분한가? (최고 점수 >= 임계값)
pub fn has_evidence(best_score: Option<f32>) -> bool {
    matches!(best_score, Some(s) if s >= RELEVANCE_MIN)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_message_includes_all_parts() {
        let m = build_user_message("컨텍스트내용", "질문내용");
        assert!(m.contains("컨텍스트내용"));
        assert!(m.contains("질문내용"));
    }

    #[test]
    fn evidence_threshold() {
        assert!(has_evidence(Some(0.9)));
        assert!(!has_evidence(Some(0.1)));
        assert!(!has_evidence(None));
    }
}
