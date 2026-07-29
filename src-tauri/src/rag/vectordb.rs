// 벡터 저장소. 기기 로컬 JSON 영속 + brute-force 코사인.
//
// 데스크탑판과 결정적 차이: **컬렉션이 1개뿐**이다.
// 모바일은 유출가능(leakable) 문서만 저장하므로 all/leakable 분리가 의미 없다.
// 파일명에 `leakable` 을 남겨 저장소의 성격을 코드에서 드러낸다.
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 단일 컬렉션 파일명 — 유출가능 문서 전용임을 이름으로 못박는다.
pub const COLLECTION_FILE: &str = "leakable.json";

#[derive(Serialize, Deserialize, Clone)]
pub struct Record {
    pub doc_id: String,
    pub chunk_index: usize,
    pub page: u32,
    pub text: String,
    pub vector: Vec<f32>,
}

#[derive(Clone, Debug)]
pub struct Hit {
    pub doc_id: String,
    pub chunk_index: usize,
    pub page: u32,
    pub text: String,
    pub score: f32,
}

/// 코사인 유사도. 영벡터/길이불일치 → 0.
pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;
    for i in 0..a.len() {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    dot / (na.sqrt() * nb.sqrt())
}

pub struct VectorStore {
    dir: PathBuf,
    records: Vec<Record>,
}

impl VectorStore {
    pub fn load(dir: PathBuf) -> Self {
        let records = std::fs::read(dir.join(COLLECTION_FILE))
            .ok()
            .and_then(|b| serde_json::from_slice::<Vec<Record>>(&b).ok())
            .unwrap_or_default();
        Self { dir, records }
    }

    fn persist(&self) -> Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        std::fs::write(
            self.dir.join(COLLECTION_FILE),
            serde_json::to_vec(&self.records)?,
        )?;
        Ok(())
    }

    /// 문서 청크 벡터 삽입. 동일 doc_id 기존 레코드는 교체(재인덱싱).
    pub fn upsert(&mut self, doc_id: &str, records: Vec<Record>) -> Result<()> {
        self.records.retain(|r| r.doc_id != doc_id);
        self.records.extend(records);
        self.persist()
    }

    /// 하이브리드 검색: 코사인(의미) + 어휘(정확 용어) 결합.
    /// 순수 벡터가 놓치는 고유명사·코드("F-5","부모초청")를 끌어올린다.
    /// score = (1-lambda)*cosine + lambda*lexical.
    pub fn search_hybrid(
        &self,
        query: &[f32],
        terms: &[String],
        top_k: usize,
        lambda: f32,
    ) -> Vec<Hit> {
        let mut scored: Vec<Hit> = self
            .records
            .iter()
            .map(|r| Hit {
                doc_id: r.doc_id.clone(),
                chunk_index: r.chunk_index,
                page: r.page,
                text: r.text.clone(),
                score: (1.0 - lambda) * cosine(query, &r.vector)
                    + lambda * lexical_score(terms, &r.text),
            })
            .collect();
        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(top_k);
        scored
    }

    pub fn delete_doc(&mut self, doc_id: &str) -> Result<()> {
        self.records.retain(|r| r.doc_id != doc_id);
        self.persist()
    }

    pub fn count(&self) -> usize {
        self.records.len()
    }

    pub fn has_doc(&self, doc_id: &str) -> bool {
        self.records.iter().any(|r| r.doc_id == doc_id)
    }
}

/// 질의 → 어휘 매칭용 용어. 공백/구두점 분리, 2자 이상, 중복 제거.
pub fn query_terms(query: &str) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    query
        .split(|c: char| c.is_whitespace() || ",.?!·:;()[]/\"'".contains(c))
        .map(|t| t.chars().filter(|c| !c.is_whitespace()).collect::<String>())
        .filter(|t| t.chars().count() >= 2)
        .filter(|t| seen.insert(t.clone()))
        .collect()
}

/// 어휘 점수: 청크에 등장하는 질의 용어 비율(0..1).
/// 한국어 띄어쓰기 편차 대비 공백 무시 매칭.
fn lexical_score(terms: &[String], text: &str) -> f32 {
    if terms.is_empty() {
        return 0.0;
    }
    let text_nospace: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    let matched = terms.iter().filter(|t| text_nospace.contains(*t)).count();
    matched as f32 / terms.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(doc: &str, idx: usize, v: Vec<f32>) -> Record {
        Record {
            doc_id: doc.into(),
            chunk_index: idx,
            page: 1,
            text: format!("{doc}-{idx}"),
            vector: v,
        }
    }

    fn tmp_dir() -> PathBuf {
        std::env::temp_dir().join(format!("vecstore-{}", uuid::Uuid::new_v4()))
    }

    #[test]
    fn cosine_identity_and_orthogonal() {
        assert!((cosine(&[1.0, 0.0], &[1.0, 0.0]) - 1.0).abs() < 1e-6);
        assert!(cosine(&[1.0, 0.0], &[0.0, 1.0]).abs() < 1e-6);
        assert_eq!(cosine(&[0.0, 0.0], &[1.0, 1.0]), 0.0);
        assert_eq!(cosine(&[1.0], &[1.0, 0.0]), 0.0); // 차원 불일치
    }

    #[test]
    fn upsert_then_search_topk() {
        let mut s = VectorStore::load(tmp_dir());
        s.upsert("a", vec![rec("a", 0, vec![1.0, 0.0])]).unwrap();
        s.upsert("b", vec![rec("b", 0, vec![0.0, 1.0])]).unwrap();
        let hits = s.search_hybrid(&[0.9, 0.1], &[], 1, 0.0);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].doc_id, "a");
    }

    #[test]
    fn reupsert_replaces_same_doc() {
        let mut s = VectorStore::load(tmp_dir());
        s.upsert(
            "a",
            vec![rec("a", 0, vec![1.0, 0.0]), rec("a", 1, vec![0.5, 0.5])],
        )
        .unwrap();
        assert_eq!(s.count(), 2);
        s.upsert("a", vec![rec("a", 0, vec![1.0, 0.0])]).unwrap();
        assert_eq!(s.count(), 1);
    }

    #[test]
    fn query_terms_extraction() {
        let t = query_terms("F-5 영주권자 부모초청 방법?");
        assert!(t.contains(&"영주권자".to_string()));
        assert!(t.contains(&"부모초청".to_string()));
        assert!(t.contains(&"F-5".to_string())); // 하이픈은 분리하지 않음
    }

    #[test]
    fn query_terms_dedups() {
        assert_eq!(query_terms("비자 비자 비자").len(), 1);
    }

    #[test]
    fn lexical_boosts_exact_term_chunk() {
        let mut s = VectorStore::load(tmp_dir());
        // A 는 쿼리벡터에 더 가깝지만 용어 없음, B 는 멀지만 "부모초청" 포함.
        let mut a = rec("a", 0, vec![1.0, 0.0]);
        a.text = "일반 안내 문서".into();
        let mut b = rec("b", 0, vec![0.0, 1.0]);
        b.text = "부모초청 우수인재 절차".into();
        s.upsert("a", vec![a]).unwrap();
        s.upsert("b", vec![b]).unwrap();

        let terms = query_terms("부모초청 우수인재");
        let hits = s.search_hybrid(&[1.0, 0.0], &terms, 2, 0.6);
        assert_eq!(hits[0].doc_id, "b", "어휘 매칭 청크가 상위여야");
    }

    #[test]
    fn lexical_ignores_whitespace_variants() {
        // 문서에 "부모 초청" 처럼 띄어쓰기가 달라도 매칭돼야 한다.
        assert_eq!(lexical_score(&["부모초청".into()], "부모 초청 절차"), 1.0);
    }

    #[test]
    fn delete_removes_doc() {
        let mut s = VectorStore::load(tmp_dir());
        s.upsert("a", vec![rec("a", 0, vec![1.0, 0.0])]).unwrap();
        assert!(s.has_doc("a"));
        s.delete_doc("a").unwrap();
        assert_eq!(s.count(), 0);
        assert!(!s.has_doc("a"));
    }

    #[test]
    fn persists_across_reload() {
        let dir = tmp_dir();
        {
            let mut s = VectorStore::load(dir.clone());
            s.upsert("a", vec![rec("a", 0, vec![1.0, 2.0])]).unwrap();
        }
        assert_eq!(VectorStore::load(dir).count(), 1);
    }
}
