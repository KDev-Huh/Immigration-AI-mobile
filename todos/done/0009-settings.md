# Task 0009 — 설정 탭

- **상태**: done
- **의존**: 0008

## 구현

- `src/tabs/SettingsTab.tsx`: 공급자 선택 / 모델 선택(datalist, 직접 입력 가능) / 키 등록·삭제.
- `src/lib/settings.ts`: provider·model 만 localStorage 영속. **API 키는 절대 localStorage 에 두지 않음**.
- 키는 화면에 되읽지 않음 — 존재 여부(`has_api_key`)만 조회.
- "임베딩은 항상 OpenAI → Anthropic 채팅이어도 OpenAI 키 필수" 안내 명시.
- `ChatTab` 은 설정을 읽기만 하고, 키 미비 시 상단에 경고 표시.

## Acceptance Criteria

- [x] 저장된 provider 값이 손상돼도 안전한 기본값(openai)으로 복구 (`isProvider` 검증)
- [x] 키 저장/삭제 후 상태 즉시 갱신
- [x] `npm test` settings 2개 통과
