# ADR 0001 — 기술 스택 (모바일, 클라우드 전용)

- **상태**: accepted
- **날짜**: 2026-07-29

## 배경

데스크탑판을 모바일(iOS/Android)로 파생. 이동 중 사용. 로컬 LLM은 모바일에서 비현실적(모델 수 GB, 연산 부담) → 제거하고 클라우드 LLM만.

## 결정

- **앱**: Tauri v2 모바일 — 데스크탑의 Rust/React 코드 최대 재사용.
- **LLM**: OpenAI API (chat + embedding). Anthropic(chat) 선택, 임베딩은 OpenAI.
- **임베딩**: `text-embedding-3-small` (1536차원, 클라우드).
- **벡터DB**: 기기 로컬(초기 JSON, 규모 크면 SQLite).
- **PDF**: pdf-extract (데스크탑 검증).

## 대안

- React Native/Flutter — Rust 코어 재사용 불가. 기각.
- 로컬 LLM 유지 — 모바일 자원 한계. 기각.
- ChatGPT Plus 브라우저 로그인 — Plus는 API 접근권 아님, 약관 위반·불안정. 유예.

## 결과

- 장점: 데스크탑 코어 대거 재사용, 가벼운 앱, 오프라인 모델 불필요.
- 트레이드오프: 네트워크 필수, 클라우드 사용량 과금, Tauri 모바일 빌드·서명 학습.
