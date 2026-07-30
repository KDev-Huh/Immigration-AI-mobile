// Tauri IPC 커맨드 — 프론트 src/lib/ipc.ts 와 1:1 대응.
// 여기서는 검증·라우팅만. 실제 로직은 각 모듈에 위임.
//
// 보안 경계: 업로드 시 유출불가 문서를 거부하는 판정이 여기 있다. 프론트 검증은 편의일 뿐,
// **최후 방어선은 upload_document 안의 allows_upload 체크**다.
use crate::documents::store::DocStore;
use crate::documents::{chunker, parser, DocumentMeta, Sensitivity, UPLOAD_REJECT};
use crate::rag::embedding;
use crate::rag::vectordb::{query_terms, VectorStore};
use crate::rag::{
    build_user_message, has_evidence, retriever, ChatAnswer, CONTEXT_BUDGET_CHARS, HYBRID_LAMBDA,
    NO_EVIDENCE, SYSTEM_PROMPT, TOP_K,
};
use crate::security::{self, CloudProvider};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

fn now_millis() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis().to_string())
        .unwrap_or_default()
}

/// 인덱싱 진행률 event 페이로드 (types.ts IndexProgress 와 계약).
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct IndexProgress<'a> {
    doc_id: &'a str,
    stage: &'a str,
    progress: f32,
}

fn emit_progress(app: &AppHandle, doc_id: &str, stage: &str, progress: f32) {
    // 진행률 전달 실패가 인덱싱 자체를 중단시키면 안 된다.
    let _ = app.emit(
        "index-progress",
        IndexProgress {
            doc_id,
            stage,
            progress,
        },
    );
}

#[tauri::command]
pub fn list_documents(store: State<'_, Mutex<DocStore>>) -> Result<Vec<DocumentMeta>, String> {
    let store = store.lock().map_err(|e| e.to_string())?;
    Ok(store.list())
}

/// 문서 업로드. 프론트가 읽어 넘긴 바이트를 앱 데이터에 복사해 소유한다.
/// 유출불가(confidential) 태그는 **거부** — 모바일은 클라우드 전송이 전제이기 때문.
#[tauri::command]
pub fn upload_document(
    filename: String,
    bytes: Vec<u8>,
    sensitivity: Sensitivity,
    store: State<'_, Mutex<DocStore>>,
) -> Result<DocumentMeta, String> {
    if !sensitivity.allows_upload() {
        return Err(UPLOAD_REJECT.to_string());
    }
    if bytes.is_empty() {
        return Err("빈 파일입니다".to_string());
    }
    // 확장자가 아니라 내용으로 판정한다 — 모바일 피커가 주는 이름에는
    // 확장자가 없을 수 있다 (content:// 문서 ID).
    parser::detect_pdf_bytes(&bytes).map_err(|e| e.to_string())?;

    let hash = format!("{:x}", Sha256::digest(&bytes));
    let meta = DocumentMeta {
        id: Uuid::new_v4().to_string(),
        filename: parser::sanitize_filename(&filename),
        sensitivity,
        pages: 0,
        chunk_count: 0,
        status: "pending".into(),
        progress: 0.0,
        updated_at: now_millis(),
        error: None,
    };

    let mut store = store.lock().map_err(|e| e.to_string())?;
    store.add(meta, hash, &bytes).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_document(
    id: String,
    store: State<'_, Mutex<DocStore>>,
    vec_store: State<'_, Mutex<VectorStore>>,
) -> Result<(), String> {
    store
        .lock()
        .map_err(|e| e.to_string())?
        .delete(&id)
        .map_err(|e| e.to_string())?;
    vec_store
        .lock()
        .map_err(|e| e.to_string())?
        .delete_doc(&id)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// 인덱싱: 파싱 → 청킹 → 클라우드 임베딩 → 벡터DB. OpenAI 키 필요.
/// 실패 시 문서를 error 상태로 남겨 재시도할 수 있게 한다.
#[tauri::command]
pub async fn index_document(
    app: AppHandle,
    id: String,
    doc_store: State<'_, Mutex<DocStore>>,
    vec_store: State<'_, Mutex<VectorStore>>,
) -> Result<DocumentMeta, String> {
    match run_index(&app, &id, &doc_store, &vec_store).await {
        Ok(meta) => {
            emit_progress(&app, &id, "ready", 1.0);
            Ok(meta)
        }
        Err(e) => {
            if let Ok(mut s) = doc_store.lock() {
                let _ = s.mark_error(&id, &e);
            }
            emit_progress(&app, &id, "error", 0.0);
            Err(e)
        }
    }
}

async fn run_index(
    app: &AppHandle,
    id: &str,
    doc_store: &State<'_, Mutex<DocStore>>,
    vec_store: &State<'_, Mutex<VectorStore>>,
) -> Result<DocumentMeta, String> {
    // 키를 먼저 확인 — 무거운 파싱을 하고 나서 키 없음으로 실패하면 낭비다.
    let api_key = off_main(security::embedding_api_key).await?;

    let path = {
        let s = doc_store.lock().map_err(|e| e.to_string())?;
        s.source_path(id).ok_or_else(|| "문서 없음".to_string())?
    };

    // 1) 파싱 — CPU 바운드라 별도 스레드에서. UI 스레드/런타임을 막지 않는다.
    set_stage(app, doc_store, id, "parsing", 0.05)?;
    let pages = tokio::task::spawn_blocking(move || parser::parse_file(&path))
        .await
        .map_err(|e| format!("파싱 작업 실패: {e}"))?
        .map_err(|e| e.to_string())?;

    // 2) 청킹
    set_stage(app, doc_store, id, "chunking", 0.15)?;
    let chunks = chunker::chunk(id, &pages, &chunker::ChunkConfig::default());
    if chunks.is_empty() {
        return Err("추출된 내용이 없어 인덱싱할 수 없습니다".to_string());
    }
    let texts: Vec<String> = chunks.iter().map(|c| c.text.clone()).collect();

    // 3) 클라우드 임베딩 — 진행률 0.2 ~ 0.95 구간에 매핑.
    set_stage(app, doc_store, id, "embedding", 0.2)?;
    let vectors =
        embedding::embed_batch_with(&api_key, &texts, embedding::DEFAULT_BATCH, |done, total| {
            let ratio = if total == 0 {
                1.0
            } else {
                done as f32 / total as f32
            };
            emit_progress(app, id, "embedding", 0.2 + 0.75 * ratio);
        })
        .await
        .map_err(|e| e.to_string())?;

    // 4) 벡터DB 반영
    let records = crate::jobs::build_records(id, &chunks, vectors).map_err(|e| e.to_string())?;
    {
        let mut vs = vec_store.lock().map_err(|e| e.to_string())?;
        vs.upsert(id, records).map_err(|e| e.to_string())?;
    }

    let mut s = doc_store.lock().map_err(|e| e.to_string())?;
    s.mark_ready(id, pages.len() as u32, chunks.len() as u32)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "문서 없음".to_string())
}

fn set_stage(
    app: &AppHandle,
    doc_store: &State<'_, Mutex<DocStore>>,
    id: &str,
    stage: &str,
    progress: f32,
) -> Result<(), String> {
    doc_store
        .lock()
        .map_err(|e| e.to_string())?
        .set_stage(id, stage, progress)
        .map_err(|e| e.to_string())?;
    emit_progress(app, id, stage, progress);
    Ok(())
}

/// RAG 질의. 검색 대상은 유출가능 문서 단일 컬렉션뿐이다 (분기 자체가 없음).
/// 쿼리 임베딩도 클라우드(OpenAI) — 데스크탑판과 다른 지점.
#[tauri::command]
pub async fn ask(
    query: String,
    provider: CloudProvider,
    model: Option<String>,
    doc_store: State<'_, Mutex<DocStore>>,
    vec_store: State<'_, Mutex<VectorStore>>,
) -> Result<ChatAnswer, String> {
    if query.trim().is_empty() {
        return Err("질문이 비어 있습니다".to_string());
    }

    // 1) 쿼리 임베딩 (OpenAI)
    let embed_key = off_main(security::embedding_api_key).await?;
    let qvec = embedding::embed_query(&embed_key, &query)
        .await
        .map_err(|e| e.to_string())?;

    // 2) 하이브리드 검색 (코사인 + 어휘)
    let terms = query_terms(&query);
    let hits = {
        let vs = vec_store.lock().map_err(|e| e.to_string())?;
        vs.search_hybrid(&qvec, &terms, TOP_K, HYBRID_LAMBDA)
    };

    // 3) 근거 부족 → 생성 생략. 지어내지 않는다.
    if !has_evidence(hits.first().map(|h| h.score)) {
        return Ok(ChatAnswer {
            text: NO_EVIDENCE.to_string(),
            citations: Vec::new(),
        });
    }

    // 4) 컨텍스트 조립 (파일명 해석)
    let names = { doc_store.lock().map_err(|e| e.to_string())?.filename_map() };
    let ctx = retriever::assemble(
        &hits,
        |id| names.get(id).cloned().unwrap_or_else(|| id.to_string()),
        CONTEXT_BUDGET_CHARS,
    );

    // 5) 생성 — 락 미보유 상태에서 await.
    let chat_key = off_main(move || security::get_api_key(provider))
        .await
        .map_err(|_| "API 키가 없습니다. 설정 탭에서 등록하세요.".to_string())?;
    let user = build_user_message(&ctx.prompt_context, &query);
    let text =
        crate::llm::cloud::generate(provider, &chat_key, SYSTEM_PROMPT, &user, model.as_deref())
            .await
            .map_err(|e| e.to_string())?;

    Ok(ChatAnswer {
        text,
        citations: ctx.citations,
    })
}

// 보안저장 커맨드는 전부 `spawn_blocking` 위에서 돌린다.
// Android 구현이 JavaVM/Activity 확보를 위해 메인 스레드로 디스패치하고 그 응답을
// 기다리므로, 메인 스레드에서 실행되면 교착한다.
async fn off_main<T, F>(work: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> anyhow::Result<T> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|e| format!("작업 실행 실패: {e}"))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_api_key(provider: CloudProvider, key: String) -> Result<(), String> {
    off_main(move || security::set_api_key(provider, &key)).await
}

#[tauri::command]
pub async fn has_api_key(provider: CloudProvider) -> Result<bool, String> {
    off_main(move || security::has_api_key(provider)).await
}

#[tauri::command]
pub async fn delete_api_key(provider: CloudProvider) -> Result<(), String> {
    off_main(move || security::delete_api_key(provider)).await
}
