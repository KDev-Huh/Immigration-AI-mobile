# Git 커밋 컨벤션

포맷: `type :: 메시지`

예) `feat :: 문서 업로드 + 유출 태깅 구현`

## 타입 (Conventional Commits 기반)

| type | 용도 |
|---|---|
| `feat` | 새 기능 |
| `fix` | 버그 수정 |
| `refactor` | 동작 변화 없는 코드 개선 |
| `docs` | 문서만 변경 (README, docs/, 기획서 등) |
| `test` | 테스트 추가/수정 |
| `chore` | 빌드·설정·의존성 등 잡무 (동작 무관) |
| `style` | 포맷·세미콜론 등 코드 의미 변화 없음 |
| `perf` | 성능 개선 |
| `build` | 빌드 시스템·외부 의존성 (Cargo, npm, tauri) |
| `ci` | CI 설정 |
| `revert` | 이전 커밋 되돌림 |

## 규칙

- 제목: `type :: ` 뒤 명령형/현재형 한 줄. ~50자 내 권장.
- 본문(선택): "왜"가 자명하지 않을 때만. 빈 줄 후 작성.
- 태스크 연계 시 본문에 `Task 0001`, `Spec 0001` 참조.
- 커밋은 논리 단위로. 무관한 변경 섞지 않기.

## 예시

```
feat :: 벡터DB 2컬렉션 분리 (all/leakable)
fix :: 청커 페이지 경계 오버랩 계산 오류
docs :: ADR 0002 보안 경계 결정 기록
test :: collection_for 클라우드 경계 단위테스트
build :: tauri v2 + keyring 의존성 추가
chore :: 하네스 폴더 구조 스캐폴드
```
