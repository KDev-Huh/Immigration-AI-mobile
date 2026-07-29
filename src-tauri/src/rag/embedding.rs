// 임베딩 오케스트레이션. 모바일은 **클라우드(OpenAI) 전용** — 로컬 임베딩 없음.
// 채팅 공급자가 Anthropic 이어도 임베딩은 OpenAI 를 쓴다 (Anthropic 임베딩 API 부재).
use anyhow::Result;

/// 임베딩 모델. 바꾸면 전체 재인덱싱 필요(차원·분포 변경).
pub const EMBED_MODEL: &str = "text-embedding-3-small";
/// text-embedding-3-small 차원 → 벡터DB 레코드와 일치해야 함.
pub const EMBED_DIM: usize = 1536;
/// 1회 요청 배치 크기. 너무 크면 요청 본문·타임아웃 위험.
pub const DEFAULT_BATCH: usize = 64;

/// 배치 경계 [start,end) 나열. 순서 보존 근거 (순수·테스트).
pub fn batch_spans(total: usize, size: usize) -> Vec<(usize, usize)> {
    let size = size.max(1);
    let mut spans = Vec::new();
    let mut s = 0;
    while s < total {
        let e = (s + size).min(total);
        spans.push((s, e));
        s = e;
    }
    spans
}

/// 배치 임베딩. 순서 보존. 진행률 콜백 (완료수, 총수).
pub async fn embed_batch_with<F: FnMut(usize, usize)>(
    api_key: &str,
    texts: &[String],
    batch: usize,
    mut progress: F,
) -> Result<Vec<Vec<f32>>> {
    let mut out = Vec::with_capacity(texts.len());
    for (s, e) in batch_spans(texts.len(), batch) {
        let mut vecs = crate::llm::cloud::embed(api_key, EMBED_MODEL, &texts[s..e]).await?;
        out.append(&mut vecs);
        progress(out.len(), texts.len());
    }
    Ok(out)
}

/// 기본 배치·무진행콜백 편의 래퍼.
pub async fn embed_batch(api_key: &str, texts: &[String]) -> Result<Vec<Vec<f32>>> {
    embed_batch_with(api_key, texts, DEFAULT_BATCH, |_, _| {}).await
}

/// 쿼리 1개 임베딩.
pub async fn embed_query(api_key: &str, query: &str) -> Result<Vec<f32>> {
    let mut v = crate::llm::cloud::embed(api_key, EMBED_MODEL, &[query.to_string()]).await?;
    v.pop()
        .ok_or_else(|| anyhow::anyhow!("쿼리 임베딩 결과가 비어 있습니다"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dim_matches_openai_small_model() {
        assert_eq!(EMBED_DIM, 1536);
        assert_eq!(EMBED_MODEL, "text-embedding-3-small");
    }

    #[test]
    fn batch_spans_divides_with_remainder() {
        assert_eq!(batch_spans(5, 2), vec![(0, 2), (2, 4), (4, 5)]);
    }

    #[test]
    fn batch_spans_empty() {
        assert_eq!(batch_spans(0, 4), Vec::<(usize, usize)>::new());
    }

    #[test]
    fn batch_spans_size_ge_total() {
        assert_eq!(batch_spans(3, 10), vec![(0, 3)]);
    }

    #[test]
    fn batch_spans_covers_all_in_order() {
        let spans = batch_spans(200, 64);
        assert_eq!(spans.first().unwrap().0, 0);
        assert_eq!(spans.last().unwrap().1, 200);
        for w in spans.windows(2) {
            assert_eq!(w[0].1, w[1].0); // 연속·무중복
        }
    }

    #[test]
    fn batch_spans_zero_size_does_not_hang() {
        assert_eq!(batch_spans(2, 0), vec![(0, 1), (1, 2)]);
    }
}
