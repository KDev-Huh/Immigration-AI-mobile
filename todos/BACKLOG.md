# 백로그 (우선순위 순)

## 1단계 — 스캐폴드 + 문서 파이프라인

- [x] 0001 프로젝트 스캐폴드 (Tauri v2 모바일 + React/TS + Rust 코어) (`done/0001-scaffold.md`)
- [x] 0002 문서 업로드(유출가능 전용) + 목록/삭제 (`done/0002-upload.md`)
- [x] 0003 PDF 파서 이식 (pdf-extract) (`done/0003-parser.md`)
- [x] 0004 청커 이식 (`done/0004-chunker.md`)

## 2단계 — 클라우드 RAG

- [x] 0005 클라우드 임베딩 (OpenAI text-embedding-3-small) (`done/0005-cloud-embedding.md`)
- [x] 0006 로컬 벡터DB (단일 컬렉션) + 하이브리드 검색 이식 (`done/0006-vectordb.md`)
- [x] 0007 인덱싱 파이프라인 (`done/0007-indexing.md`)
- [x] 0008 채팅 RAG (`done/0008-chat-rag.md`)

## 3단계 — 설정·보안·배포

- [x] 0009 설정 탭: 공급자/모델 선택 + API 키 입력 (`done/0009-settings.md`)
- [x] 0010 모바일 보안저장 (iOS Keychain/Android Keystore) (`done/0010-secure-store.md`)
- [x] 0011 모바일 반응형 UI (`done/0011-mobile-ui.md`)
- [x] 0012 GitHub 레포 + Actions CI/빌드 (`done/0012-ci-deploy.md`)

## 남은 일 (계정·권한 필요 — 코드 작업 아님)

- [x] 0013 CI 실행 (public 전환으로 해결)
- [x] 0016 iOS Keychain 실동작 확인 (시뮬레이터에서 set→has→복호화→delete 전 경로)
- [ ] 0017 실제 OpenAI 키로 업로드→인덱싱→질의 E2E
- [ ] 0014 Apple Developer 서명 (`APPLE_*` 시크릿) → iOS TestFlight
- [ ] 0015 Play Console 업로드 키 (`ANDROID_*` 시크릿) → 내부 테스트 트랙
