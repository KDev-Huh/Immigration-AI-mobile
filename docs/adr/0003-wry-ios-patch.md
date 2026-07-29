# ADR 0003 — wry iOS 크래시 회피를 위한 벤더 패치

- **상태**: 채택 (한시적)
- **일자**: 2026-07-29

## 맥락

iOS 시뮬레이터에서 앱이 **실행 즉시 죽었다**. 크래시 리포트:

```
EXC_BREAKPOINT (SIGTRAP)
CoreFoundation  CFRelease.cold.2
CoreFoundation  _CFBundleGetBundleWithIdentifier
Foundation      +[NSBundle bundleWithIdentifier:]
wry             platform_webview_version
tauri_runtime_wry  Wry::init
immigration_ai_mobile_lib::run
```

원인은 `wry 0.55.1` 의 `src/wkwebview/mod.rs::platform_webview_version()`:

```rust
let Some(bundle) = NSBundle::bundleWithIdentifier(ns_string!("com.apple.WebKit")) else { ... };
...
bundle.unload();   // ← 문제
```

`bundleWithIdentifier:` 가 돌려주는 번들은 CoreFoundation 이 캐싱·소유하는 객체다.
호출자가 소유권을 갖지 않으므로 `unload()` 로 해제하면 과다해제가 되고, 다음 CFRelease 에서
`EXC_BREAKPOINT` 로 죽는다.

이 함수는 `tauri-runtime-wry` 의 `Wry::init` 에서 `webview_runtime_installed` 판정을 위해
**무조건** 호출된다. 우회 설정이 없다 → iOS 빌드 전체가 실행 불가.

업스트림 `tauri-apps/wry` dev 브랜치에도 2026-07-29 기준 수정이 없다.

## 결정

`src-tauri/vendor/wry` 에 0.55.1 사본을 두고 `bundle.unload();` 한 줄만 제거한 뒤,
`src-tauri/Cargo.toml` 에서 `[patch.crates-io]` 로 연결한다.

```toml
[patch.crates-io]
wry = { path = "vendor/wry" }
```

## 대안과 기각 사유

| 대안 | 기각 사유 |
|---|---|
| 업스트림 수정 대기 | iOS 가 아예 실행 불가 — 대기할 수 없음 |
| GitHub 에 wry 포크 후 git patch | 별도 레포 수명에 빌드가 의존. 벤더 사본이 자기완결적 |
| iOS 포기 | 요구사항이 iOS/Android 양쪽 |

## 결과

- 벤더 디렉토리 764KB(예제·락파일 제거). 변경은 주석 포함 1개소뿐이라 diff 추적이 쉽다.
- **업스트림 수정이 릴리스되면 `vendor/wry` 와 `[patch.crates-io]` 블록을 함께 삭제할 것.**
  `cargo update -p wry` 후 iOS 실행이 정상이면 제거 가능.
- 벤더 코드는 우리 clippy/fmt 게이트 대상이 아니다(의존성 취급).
