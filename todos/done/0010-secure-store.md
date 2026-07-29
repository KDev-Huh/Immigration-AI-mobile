# Task 0010 — 모바일 보안저장

- **상태**: done
- **의존**: 0009

## 구현

`src-tauri/src/security/store/` 를 타깃별로 분기. 인터페이스는 `set/get/exists/delete` 4개로 통일.

| 타깃 | 백엔드 |
|---|---|
| iOS | `security-framework` → Keychain Services (generic password) |
| Android | JNI → AndroidKeyStore AES-256/GCM 마스터 키로 암호화 후 앱 전용 SharedPreferences |
| 데스크탑 | `keyring` (macOS Keychain / Windows Credential Manager / Linux keyutils) |

## Android 설계 근거

- `androidx.security:security-crypto`(EncryptedSharedPreferences)를 쓰면 gradle 의존성 추가가 필요하고,
  `gen/` 은 CI 에서 매번 재생성되므로 유지가 취약하다.
  → **프레임워크 API 만으로** 동일 보안 수준 구현: KeyStore 의 마스터 키는 내보내기 불가이므로
  prefs 파일이 유출돼도 복호화 불가.
- 저장 포맷 `base64(iv).base64(ciphertext)` (GCM 이라 IV 는 비밀 아님).
- `apply()` 대신 `commit()` — 저장 직후 `has_api_key` 조회가 즉시 이어지므로 동기 기록 필요.
- 사용자 인증(생체) 요구는 걸지 않음 — 백그라운드 인덱싱 중 잠금 화면이면 복호화가 막힌다.

## 절대 규칙

- 백엔드 확보 실패 시 **저장하지 않고 에러**. 평문 폴백 경로 없음.

## Acceptance Criteria

- [x] `cargo check --target aarch64-linux-android` 통과 (JNI 코드 포함)
- [x] `cargo check --target aarch64-apple-ios` 통과
- [x] 공백 키 저장 거부
- [x] 삭제는 멱등 (없어도 성공)
- [ ] **실기기 검증 미완** — 에뮬레이터/실기기에서 저장→재시작→조회 확인 필요
