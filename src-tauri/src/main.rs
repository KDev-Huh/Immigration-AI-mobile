// 데스크탑 바이너리 엔트리 (개발·테스트용). 모바일은 lib.rs 의 mobile_entry_point 사용.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    immigration_ai_mobile_lib::run()
}
