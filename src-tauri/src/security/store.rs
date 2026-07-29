// 플랫폼별 보안저장 디스패치. 각 백엔드는 동일한 4개 함수(set/get/exists/delete)를 제공한다.
#[cfg(target_os = "android")]
mod android;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod desktop;
#[cfg(target_os = "ios")]
mod ios;

#[cfg(target_os = "android")]
pub use android::{delete, exists, get, set};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub use desktop::{delete, exists, get, set};
#[cfg(target_os = "ios")]
pub use ios::{delete, exists, get, set};
