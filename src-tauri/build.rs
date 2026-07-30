fn main() {
    // Android 16KB 페이지 크기 대응.
    //
    // Android 15+ 부터 페이지 크기가 16KB 인 기기가 있다(Pixel 8/9 이후, `ps16k` 시스템 이미지).
    // 기본 4KB 정렬로 링크된 .so 는 그런 기기에서 로드 자체가 실패한다:
    //
    //   java.lang.UnsatisfiedLinkError: dlopen failed:
    //   empty/missing DT_HASH/DT_GNU_HASH in "libimmigration_ai_mobile_lib.so"
    //   (new hash type from the future?)
    //
    // 메시지는 해시를 가리키지만 실제 원인은 세그먼트 정렬이다. 4KB 이미지에서는 멀쩡히
    // 돌아가서 놓치기 쉽다. Play Store 도 Android 15+ 타깃 앱에 16KB 지원을 요구한다.
    // 16KB 로 정렬해도 4KB 기기에서 그대로 동작한다(상위 호환).
    //
    // `.cargo/config.toml` 의 rustflags 로는 안 된다 — tauri CLI 가 안드로이드 빌드에서
    // `RUSTFLAGS` 환경변수를 직접 세팅하고, 그러면 config 쪽 값이 통째로 무시된다.
    // 빌드 스크립트의 link-arg 는 그 영향을 받지 않는다.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android") {
        println!("cargo:rustc-link-arg=-Wl,-z,max-page-size=16384");
    }

    tauri_build::build()
}
