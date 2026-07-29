// 문서 메타 영속 + 원본 파일 보관. 전부 기기 로컬(앱 데이터 디렉토리).
// 데스크탑판과 차이: 모바일 파일 피커 경로는 재접근이 보장되지 않으므로
// 업로드 시점에 **바이트를 앱 데이터로 복사**해 소유한다.
use crate::documents::DocumentMeta;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 내부 레코드 = 프론트 메타 + 저장 부가정보(해시). 해시는 프론트 미노출.
#[derive(Serialize, Deserialize, Clone)]
struct StoredDoc {
    #[serde(flatten)]
    meta: DocumentMeta,
    hash: String,
}

pub struct DocStore {
    dir: PathBuf,
    docs: Vec<StoredDoc>,
}

impl DocStore {
    /// `dir/documents.json` 로드 + `dir/files/` 를 원본 보관소로 사용.
    pub fn load(dir: PathBuf) -> Self {
        let docs = std::fs::read(dir.join("documents.json"))
            .ok()
            .and_then(|b| serde_json::from_slice::<Vec<StoredDoc>>(&b).ok())
            .unwrap_or_default();
        Self { dir, docs }
    }

    fn files_dir(&self) -> PathBuf {
        self.dir.join("files")
    }

    /// 문서 원본 파일 경로 (확장자는 PDF 고정 — 현재 PDF만 지원).
    fn file_path(&self, id: &str) -> PathBuf {
        self.files_dir().join(format!("{id}.pdf"))
    }

    fn persist(&self) -> Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        std::fs::write(
            self.dir.join("documents.json"),
            serde_json::to_vec_pretty(&self.docs)?,
        )?;
        Ok(())
    }

    /// 등록 + 원본 저장. 동일 내용(해시)이 이미 있으면 거부 — 중복 업로드 금지.
    pub fn add(&mut self, meta: DocumentMeta, hash: String, bytes: &[u8]) -> Result<DocumentMeta> {
        if let Some(existing) = self.docs.iter().find(|d| d.hash == hash) {
            return Err(anyhow!(
                "이미 업로드된 파일입니다: '{}'",
                existing.meta.filename
            ));
        }
        std::fs::create_dir_all(self.files_dir())?;
        std::fs::write(self.file_path(&meta.id), bytes)?;
        self.docs.push(StoredDoc {
            meta: meta.clone(),
            hash,
        });
        self.persist()?;
        Ok(meta)
    }

    pub fn list(&self) -> Vec<DocumentMeta> {
        self.docs.iter().map(|d| d.meta.clone()).collect()
    }

    /// doc_id → 파일명 맵 (출처 표시용).
    pub fn filename_map(&self) -> std::collections::HashMap<String, String> {
        self.docs
            .iter()
            .map(|d| (d.meta.id.clone(), d.meta.filename.clone()))
            .collect()
    }

    /// 인덱싱 대상 원본 경로. 등록되지 않은 id 면 None.
    pub fn source_path(&self, id: &str) -> Option<PathBuf> {
        self.docs
            .iter()
            .find(|d| d.meta.id == id)
            .map(|d| self.file_path(&d.meta.id))
    }

    fn find_mut(&mut self, id: &str) -> Option<&mut StoredDoc> {
        self.docs.iter_mut().find(|d| d.meta.id == id)
    }

    /// 진행 단계 갱신 (parsing/chunking/embedding).
    pub fn set_stage(&mut self, id: &str, status: &str, progress: f32) -> Result<()> {
        if let Some(d) = self.find_mut(id) {
            d.meta.status = status.to_string();
            d.meta.progress = progress;
            self.persist()?;
        }
        Ok(())
    }

    /// 인덱싱 완료: status=ready + 페이지/청크수 갱신.
    pub fn mark_ready(
        &mut self,
        id: &str,
        pages: u32,
        chunk_count: u32,
    ) -> Result<Option<DocumentMeta>> {
        match self.find_mut(id) {
            Some(d) => {
                d.meta.status = "ready".into();
                d.meta.progress = 1.0;
                d.meta.pages = pages;
                d.meta.chunk_count = chunk_count;
                d.meta.error = None;
                let meta = d.meta.clone();
                self.persist()?;
                Ok(Some(meta))
            }
            None => Ok(None),
        }
    }

    /// 인덱싱 실패 기록. 문서는 남기고 재시도 가능 상태로 둔다.
    pub fn mark_error(&mut self, id: &str, message: &str) -> Result<()> {
        if let Some(d) = self.find_mut(id) {
            d.meta.status = "error".into();
            d.meta.progress = 0.0;
            d.meta.error = Some(message.to_string());
            self.persist()?;
        }
        Ok(())
    }

    /// 삭제 — 메타 + 원본 파일 모두 제거.
    pub fn delete(&mut self, id: &str) -> Result<bool> {
        let Some(pos) = self.docs.iter().position(|d| d.meta.id == id) else {
            return Ok(false);
        };
        let path = self.file_path(id);
        self.docs.remove(pos);
        remove_if_exists(&path);
        self.persist()?;
        Ok(true)
    }
}

fn remove_if_exists(path: &Path) {
    // 원본이 이미 없어도 삭제는 성공으로 취급 (메타 제거가 본질).
    let _ = std::fs::remove_file(path);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::Sensitivity;

    fn meta(id: &str) -> DocumentMeta {
        DocumentMeta {
            id: id.into(),
            filename: format!("{id}.pdf"),
            sensitivity: Sensitivity::Leakable,
            pages: 0,
            chunk_count: 0,
            status: "pending".into(),
            progress: 0.0,
            updated_at: "0".into(),
            error: None,
        }
    }

    fn tmp() -> PathBuf {
        std::env::temp_dir().join(format!("docstore-{}", uuid::Uuid::new_v4()))
    }

    #[test]
    fn add_stores_bytes_and_lists() {
        let dir = tmp();
        let mut s = DocStore::load(dir.clone());
        s.add(meta("a"), "h1".into(), b"%PDF-1.4 fake").unwrap();
        assert_eq!(s.list().len(), 1);
        let p = s.source_path("a").unwrap();
        assert_eq!(std::fs::read(p).unwrap(), b"%PDF-1.4 fake");
    }

    #[test]
    fn rejects_duplicate_hash() {
        let mut s = DocStore::load(tmp());
        s.add(meta("a"), "same".into(), b"x").unwrap();
        assert!(s.add(meta("b"), "same".into(), b"x").is_err());
        assert_eq!(s.list().len(), 1);
    }

    #[test]
    fn delete_removes_meta_and_file() {
        let mut s = DocStore::load(tmp());
        s.add(meta("a"), "h".into(), b"x").unwrap();
        let path = s.source_path("a").unwrap();
        assert!(s.delete("a").unwrap());
        assert!(s.list().is_empty());
        assert!(!path.exists());
        assert!(!s.delete("a").unwrap()); // 두 번째는 false
    }

    #[test]
    fn mark_ready_and_error() {
        let mut s = DocStore::load(tmp());
        s.add(meta("a"), "h".into(), b"x").unwrap();
        let m = s.mark_ready("a", 3, 12).unwrap().unwrap();
        assert_eq!(m.status, "ready");
        assert_eq!(m.pages, 3);
        assert_eq!(m.chunk_count, 12);

        s.mark_error("a", "임베딩 실패").unwrap();
        assert_eq!(s.list()[0].status, "error");
        assert_eq!(s.list()[0].error.as_deref(), Some("임베딩 실패"));
    }

    #[test]
    fn persists_across_reload() {
        let dir = tmp();
        {
            let mut s = DocStore::load(dir.clone());
            s.add(meta("a"), "h".into(), b"x").unwrap();
        }
        let s2 = DocStore::load(dir);
        assert_eq!(s2.list().len(), 1);
        assert_eq!(s2.list()[0].sensitivity, Sensitivity::Leakable);
    }
}
