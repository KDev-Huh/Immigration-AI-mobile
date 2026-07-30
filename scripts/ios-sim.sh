#!/usr/bin/env bash
# 시뮬레이터에서 앱 실행 (핫 리로드). Apple Developer 팀이 필요 없다.
#
# 왜 이 스크립트가 필요한가:
#   `tauri ios dev` 를 기기 지정 없이 실행하면 연결된 **실기기**를 골라버리고,
#   실기기 빌드는 서명을 요구해서 이렇게 실패한다:
#     error: Signing for "..._iOS" requires a development team.
#   시뮬레이터를 이름으로 명시하면 xcodebuild 가 `-sdk iphonesimulator` 로 빌드하므로
#   서명 자체가 생략된다.
#
# 사용법:
#   scripts/ios-sim.sh                # 부팅된 시뮬레이터, 없으면 기본 후보를 부팅
#   scripts/ios-sim.sh "iPhone 17"    # 이름 직접 지정
set -euo pipefail

pick_booted() {
  xcrun simctl list devices booted -j 2>/dev/null | python3 -c '
import json, sys
data = json.load(sys.stdin).get("devices", {})
for runtime, devices in data.items():
    if "iOS" not in runtime:
        continue
    for d in devices:
        print(d["name"])
        raise SystemExit
'
}

pick_available() {
  xcrun simctl list devices available -j | python3 -c '
import json, sys
data = json.load(sys.stdin).get("devices", {})
best = None
for runtime, devices in data.items():
    if "iOS" not in runtime:
        continue
    for d in devices:
        if d["name"].startswith("iPhone"):
            best = best or d["name"]
print(best or "")
'
}

# vite 는 strictPort 라서 1420 이 점유돼 있으면 beforeDevCommand 가 죽는다.
# 그러면 Xcode 의 "Build Rust Code" 단계가 tauri CLI 에 붙지 못해
# `failed to read CLI options: ... ConnectionRefused` 라는 엉뚱한 패닉으로 끝난다.
# 원인을 알아볼 수 없는 에러라 여기서 미리 잡아준다.
if lsof -ti tcp:1420 >/dev/null 2>&1; then
  PIDS="$(lsof -ti tcp:1420 | tr '\n' ' ')"
  echo "포트 1420 이 이미 사용 중입니다 (pid: $PIDS)." >&2
  echo "이전 dev 서버가 남아 있습니다. 정리 후 다시 실행하세요:" >&2
  echo "  kill $PIDS" >&2
  exit 1
fi

NAME="${1:-}"
[ -n "$NAME" ] || NAME="$(pick_booted)"
[ -n "$NAME" ] || NAME="$(pick_available)"

if [ -z "$NAME" ]; then
  echo "사용 가능한 iOS 시뮬레이터가 없습니다. Xcode > Settings > Platforms 에서 iOS 런타임을 설치하세요." >&2
  exit 1
fi

echo "시뮬레이터: $NAME (서명 불필요)"
xcrun simctl boot "$NAME" 2>/dev/null || true
xcrun simctl bootstatus "$NAME" -b >/dev/null 2>&1 || true
open -a Simulator 2>/dev/null || true

exec npm run tauri -- ios dev "$NAME"
