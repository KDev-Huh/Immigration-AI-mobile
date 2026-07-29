// 앱 엔트리. 모듈 등록 + Tauri 커맨드 핸들러 배선.
// 모바일판: 로컬 LLM(Ollama) 모듈 없음. 임베딩·생성 모두 클라우드.
pub mod commands;
pub mod documents;
pub mod jobs;
pub mod llm;
pub mod rag;
pub mod security;

use std::sync::Mutex;
use tauri::Manager;

/// 모바일 엔트리포인트. Android/iOS 는 이 심볼을 호출한다.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // 문서·벡터 전부 앱 데이터 디렉토리(기기 로컬). 원격 저장소 없음.
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir).ok();
            app.manage(Mutex::new(documents::store::DocStore::load(dir.clone())));
            app.manage(Mutex::new(rag::vectordb::VectorStore::load(
                dir.join("vectors"),
            )));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_documents,
            commands::upload_document,
            commands::index_document,
            commands::delete_document,
            commands::ask,
            commands::set_api_key,
            commands::has_api_key,
            commands::delete_api_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
