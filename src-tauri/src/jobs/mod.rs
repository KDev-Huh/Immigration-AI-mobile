// 인덱싱 파이프라인 보조: 청크 + 벡터 → 벡터DB 레코드.
// 순수 로직이라 단위 테스트로 계약(개수·차원 일치)을 못박는다.
use crate::documents::Chunk;
use crate::rag::embedding::EMBED_DIM;
use crate::rag::vectordb::Record;
use anyhow::{anyhow, Result};

/// 청크 ↔ 벡터 1:1 결합. 개수·차원 불일치는 출처 오염으로 이어지므로 즉시 에러.
pub fn build_records(
    doc_id: &str,
    chunks: &[Chunk],
    vectors: Vec<Vec<f32>>,
) -> Result<Vec<Record>> {
    if chunks.len() != vectors.len() {
        return Err(anyhow!(
            "청크·벡터 개수 불일치: 청크 {}, 벡터 {}",
            chunks.len(),
            vectors.len()
        ));
    }
    if let Some(bad) = vectors.iter().find(|v| v.len() != EMBED_DIM) {
        return Err(anyhow!(
            "임베딩 차원 불일치: 기대 {EMBED_DIM}, 실제 {}",
            bad.len()
        ));
    }
    Ok(chunks
        .iter()
        .zip(vectors)
        .map(|(c, v)| Record {
            doc_id: doc_id.to_string(),
            chunk_index: c.index,
            page: c.page,
            text: c.text.clone(),
            vector: v,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk(index: usize, page: u32) -> Chunk {
        Chunk {
            doc_id: "d".into(),
            index,
            text: format!("chunk-{index}"),
            page,
        }
    }

    fn vec_ok() -> Vec<f32> {
        vec![0.1; EMBED_DIM]
    }

    #[test]
    fn pairs_chunks_with_vectors_in_order() {
        let chunks = vec![chunk(0, 1), chunk(1, 2)];
        let recs = build_records("d", &chunks, vec![vec_ok(), vec_ok()]).unwrap();
        assert_eq!(recs.len(), 2);
        assert_eq!(recs[0].chunk_index, 0);
        assert_eq!(recs[1].page, 2);
        assert_eq!(recs[1].text, "chunk-1");
    }

    #[test]
    fn count_mismatch_is_error() {
        assert!(build_records("d", &[chunk(0, 1)], vec![]).is_err());
    }

    #[test]
    fn wrong_dimension_is_error() {
        // 로컬 임베딩(1024) 벡터가 섞여 들어오는 사고를 막는다.
        let err = build_records("d", &[chunk(0, 1)], vec![vec![0.1; 1024]]);
        assert!(err.is_err());
    }
}
