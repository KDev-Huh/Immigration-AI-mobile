# vendor/

크레이트 벤더 사본. **직접 수정한 코드가 들어 있으므로 함부로 갱신하지 말 것.**

| 크레이트 | 이유 | 제거 조건 |
|---|---|---|
| `wry` 0.55.1 | `platform_webview_version()` 이 시스템 소유 번들에 `unload()` 를 호출해 iOS 앱이 실행 즉시 크래시. 그 한 줄만 제거했다 (`src/wkwebview/mod.rs`). | 업스트림 수정 릴리스 후 `[patch.crates-io]` 와 함께 삭제 |

근거: [`docs/adr/0003-wry-ios-patch.md`](../../docs/adr/0003-wry-ios-patch.md)

패치 지점을 찾으려면:

```bash
grep -rn "PATCHED (Immigration-AI-mobile)" src-tauri/vendor/
```
