// Tauri IPC 래퍼 — 모든 백엔드 호출은 여기 경유.
// Rust 쪽 commands.rs 의 #[tauri::command] 와 1:1 대응.
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  ChatAnswer,
  CloudProvider,
  DocumentMeta,
  IndexProgress,
  Sensitivity,
} from "@/types";

// --- 문서 관리 ---
export const listDocuments = () => invoke<DocumentMeta[]>("list_documents");

/**
 * 문서 업로드. 모바일 파일 피커는 content:// URI 를 돌려주므로 Rust 가 경로를
 * 직접 읽을 수 없다 → 프론트에서 바이트를 읽어 전달하고 Rust 가 앱 데이터에 저장.
 * `sensitivity` 가 confidential 이면 Rust 가 거부한다 (보안 경계).
 */
export const uploadDocument = (
  filename: string,
  bytes: Uint8Array,
  sensitivity: Sensitivity,
) =>
  invoke<DocumentMeta>("upload_document", {
    filename,
    bytes: Array.from(bytes),
    sensitivity,
  });

/** 인덱싱 실행 (파싱→청킹→클라우드 임베딩→벡터DB). OpenAI 키 필요. */
export const indexDocument = (id: string) =>
  invoke<DocumentMeta>("index_document", { id });

export const deleteDocument = (id: string) =>
  invoke<void>("delete_document", { id });

/** 인덱싱 진행률 구독. 해제 함수 반환. */
export const onIndexProgress = (cb: (p: IndexProgress) => void) =>
  listen<IndexProgress>("index-progress", (e) => cb(e.payload));

// --- 채팅 ---
/** RAG 질의. 검색 대상은 항상 유출가능 문서 단일 컬렉션. */
export const ask = (query: string, provider: CloudProvider, model?: string) =>
  invoke<ChatAnswer>("ask", { query, provider, model: model ?? null });

// --- 자격증명 (모바일 보안저장) ---
export const setApiKey = (provider: CloudProvider, key: string) =>
  invoke<void>("set_api_key", { provider, key });

export const hasApiKey = (provider: CloudProvider) =>
  invoke<boolean>("has_api_key", { provider });

export const deleteApiKey = (provider: CloudProvider) =>
  invoke<void>("delete_api_key", { provider });
