# Task 0011 — 모바일 반응형 UI

- **상태**: done
- **의존**: 0010

## 구현

- `src/App.tsx`: 데스크탑의 상단 탭바 → **하단 탭바** 3개(문서/채팅/설정). 모바일 관례.
  - 탭 전환 시 언마운트 대신 `hidden` 토글 — 채팅 입력·스크롤 상태 유지.
- `src/components/ChatPane.tsx`: 데스크탑의 상시 사이드바 → **오버레이 드로어**.
  - Enter = 줄바꿈, 전송은 버튼 (모바일 키보드 관례). Cmd/Ctrl+Enter 도 전송.
- `src/styles.css`:
  - `env(safe-area-inset-*)` 로 노치·홈 인디케이터 회피
  - 터치 타깃 44px 최소
  - `font-size: 16px` — iOS 입력 포커스 시 자동 확대 방지 임계값
  - 라이트/다크 `prefers-color-scheme`
  - 720px 이상에서 본문 폭 제한 (태블릿)
- `index.html`: `viewport-fit=cover`, `user-scalable=no`.

## Acceptance Criteria

- [x] 3탭 렌더 + 전환
- [x] safe-area 대응
- [x] 다크모드 대응
