// 공용 타입 (프론트 ↔ Rust IPC 계약).
// 데스크탑판과 달리 로컬 LLM 관련 타입(Backend/ModelInfo/PullProgress)은 없음.

/**
 * 유출 분류 태그.
 * 모바일은 클라우드 전송이 전제 → `leakable` 만 업로드 가능.
 * `confidential` 값은 "거부 대상"을 표현하기 위해 남겨둔다 (UI 경고 + Rust 거부).
 */
export type Sensitivity = "leakable" | "confidential";

export type DocStatus =
  | "pending"
  | "parsing"
  | "chunking"
  | "embedding"
  | "ready"
  | "error";

export interface DocumentMeta {
  id: string;
  filename: string;
  sensitivity: Sensitivity;
  pages: number;
  chunkCount: number;
  status: DocStatus;
  progress: number; // 0..1
  updatedAt: string;
  error?: string;
}

/** 클라우드 공급자. 브라우저 로그인은 미지원(HANDOFF 제약 참조). */
export type CloudProvider = "openai" | "anthropic";

export interface Citation {
  docId: string;
  filename: string;
  page: number;
  snippet: string;
}

export interface ChatAnswer {
  text: string;
  citations: Citation[];
}

// --- 대화 히스토리 ---
export interface Message {
  role: "user" | "assistant";
  content: string;
  citations?: Citation[];
}

export interface ChatSession {
  id: string;
  title: string;
  messages: Message[];
  createdAt: number;
}

/** 인덱싱 진행률 event 페이로드 (`index-progress`). */
export interface IndexProgress {
  docId: string;
  stage: DocStatus;
  progress: number; // 0..1
}
