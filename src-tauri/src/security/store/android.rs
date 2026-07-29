// Android 보안저장.
//
// 전략: 하드웨어 지원 AndroidKeyStore 에 AES-256/GCM 마스터 키를 만들고(내보내기 불가),
// 그 키로 API 키를 암호화해 앱 전용 SharedPreferences 에 base64 로 보관한다.
// 마스터 키는 프로세스 밖으로 나올 수 없으므로 prefs 파일이 유출돼도 복호화되지 않는다.
// androidx.security 같은 추가 gradle 의존성 없이 프레임워크 API 만 쓴다.
use crate::security::SERVICE;
use anyhow::{anyhow, Result};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use jni::objects::{JByteArray, JObject, JString, JValue};
use jni::{JNIEnv, JavaVM};

/// 마스터 키 별칭 — KeyStore 안에서만 의미. 지우면 저장된 키 전부 복호화 불가.
const KEY_ALIAS: &str = "immigration_ai_master_key";
const KEYSTORE: &str = "AndroidKeyStore";
const TRANSFORM: &str = "AES/GCM/NoPadding";
const GCM_TAG_BITS: i32 = 128;

// android.security.keystore.KeyProperties
const PURPOSE_ENCRYPT: i32 = 1;
const PURPOSE_DECRYPT: i32 = 2;
// android.content.Context.MODE_PRIVATE
const MODE_PRIVATE: i32 = 0;
// javax.crypto.Cipher
const ENCRYPT_MODE: i32 = 1;
const DECRYPT_MODE: i32 = 2;

/// SharedPreferences 파일명. 앱 전용 디렉토리에 생성된다.
fn prefs_name() -> String {
    format!("{SERVICE}.secure")
}

/// JNI 환경 + Activity Context 확보 후 클로저 실행.
fn with_env<T>(f: impl FnOnce(&mut JNIEnv, &JObject) -> Result<T>) -> Result<T> {
    let ctx = ndk_context::android_context();
    let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }
        .map_err(|e| anyhow!("JavaVM 접근 실패: {e}"))?;
    let mut guard = vm
        .attach_current_thread()
        .map_err(|e| anyhow!("JNI attach 실패: {e}"))?;
    let context = unsafe { JObject::from_raw(ctx.context().cast()) };

    let result = f(&mut guard, &context);

    // Java 예외가 남아 있으면 이후 JNI 호출이 전부 실패하므로 반드시 정리.
    if guard.exception_check().unwrap_or(false) {
        let _ = guard.exception_describe();
        let _ = guard.exception_clear();
    }
    result
}

/// AndroidKeyStore 마스터 키 확보 (없으면 생성).
fn master_key<'a>(env: &mut JNIEnv<'a>) -> Result<JObject<'a>> {
    let store_name = env.new_string(KEYSTORE)?;
    let ks = env
        .call_static_method(
            "java/security/KeyStore",
            "getInstance",
            "(Ljava/lang/String;)Ljava/security/KeyStore;",
            &[JValue::Object(&store_name)],
        )?
        .l()?;
    env.call_method(
        &ks,
        "load",
        "(Ljava/security/KeyStore$LoadStoreParameter;)V",
        &[JValue::Object(&JObject::null())],
    )?;

    let alias = env.new_string(KEY_ALIAS)?;
    let exists = env
        .call_method(
            &ks,
            "containsAlias",
            "(Ljava/lang/String;)Z",
            &[JValue::Object(&alias)],
        )?
        .z()?;
    if !exists {
        generate_master_key(env)?;
    }

    let alias = env.new_string(KEY_ALIAS)?;
    Ok(env
        .call_method(
            &ks,
            "getKey",
            "(Ljava/lang/String;[C)Ljava/security/Key;",
            &[JValue::Object(&alias), JValue::Object(&JObject::null())],
        )?
        .l()?)
}

/// 내보내기 불가 AES-GCM 키 생성. 사용자 인증 요구는 걸지 않는다
/// (백그라운드 인덱싱 중 잠금 화면이면 복호화가 막히므로).
fn generate_master_key(env: &mut JNIEnv) -> Result<()> {
    let aes = env.new_string("AES")?;
    let provider = env.new_string(KEYSTORE)?;
    let generator = env
        .call_static_method(
            "javax/crypto/KeyGenerator",
            "getInstance",
            "(Ljava/lang/String;Ljava/lang/String;)Ljavax/crypto/KeyGenerator;",
            &[JValue::Object(&aes), JValue::Object(&provider)],
        )?
        .l()?;

    let alias = env.new_string(KEY_ALIAS)?;
    let builder = env.new_object(
        "android/security/keystore/KeyGenParameterSpec$Builder",
        "(Ljava/lang/String;I)V",
        &[
            JValue::Object(&alias),
            JValue::Int(PURPOSE_ENCRYPT | PURPOSE_DECRYPT),
        ],
    )?;

    let gcm = env.new_string("GCM")?;
    let modes = env.new_object_array(1, "java/lang/String", &gcm)?;
    let builder = env
        .call_method(
            &builder,
            "setBlockModes",
            "([Ljava/lang/String;)Landroid/security/keystore/KeyGenParameterSpec$Builder;",
            &[JValue::Object(&modes)],
        )?
        .l()?;

    let nopad = env.new_string("NoPadding")?;
    let paddings = env.new_object_array(1, "java/lang/String", &nopad)?;
    let builder = env
        .call_method(
            &builder,
            "setEncryptionPaddings",
            "([Ljava/lang/String;)Landroid/security/keystore/KeyGenParameterSpec$Builder;",
            &[JValue::Object(&paddings)],
        )?
        .l()?;

    let builder = env
        .call_method(
            &builder,
            "setKeySize",
            "(I)Landroid/security/keystore/KeyGenParameterSpec$Builder;",
            &[JValue::Int(256)],
        )?
        .l()?;

    let spec = env
        .call_method(
            &builder,
            "build",
            "()Landroid/security/keystore/KeyGenParameterSpec;",
            &[],
        )?
        .l()?;

    env.call_method(
        &generator,
        "init",
        "(Ljava/security/spec/AlgorithmParameterSpec;)V",
        &[JValue::Object(&spec)],
    )?;
    env.call_method(&generator, "generateKey", "()Ljavax/crypto/SecretKey;", &[])?;
    Ok(())
}

fn shared_prefs<'a>(env: &mut JNIEnv<'a>, context: &JObject) -> Result<JObject<'a>> {
    let name = env.new_string(prefs_name())?;
    Ok(env
        .call_method(
            context,
            "getSharedPreferences",
            "(Ljava/lang/String;I)Landroid/content/SharedPreferences;",
            &[JValue::Object(&name), JValue::Int(MODE_PRIVATE)],
        )?
        .l()?)
}

fn read_pref(env: &mut JNIEnv, context: &JObject, account: &str) -> Result<Option<String>> {
    let prefs = shared_prefs(env, context)?;
    let key = env.new_string(account)?;
    let value = env
        .call_method(
            &prefs,
            "getString",
            "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
            &[JValue::Object(&key), JValue::Object(&JObject::null())],
        )?
        .l()?;
    if value.is_null() {
        return Ok(None);
    }
    let s: String = env.get_string(&JString::from(value))?.into();
    Ok(Some(s))
}

fn write_pref(env: &mut JNIEnv, context: &JObject, account: &str, value: &str) -> Result<()> {
    let prefs = shared_prefs(env, context)?;
    let editor = env
        .call_method(
            &prefs,
            "edit",
            "()Landroid/content/SharedPreferences$Editor;",
            &[],
        )?
        .l()?;
    let k = env.new_string(account)?;
    let v = env.new_string(value)?;
    env.call_method(
        &editor,
        "putString",
        "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;",
        &[JValue::Object(&k), JValue::Object(&v)],
    )?;
    // commit(): 동기 기록. 저장 직후 has_api_key 가 곧바로 조회되므로 apply() 대신 사용.
    env.call_method(&editor, "commit", "()Z", &[])?;
    Ok(())
}

fn remove_pref(env: &mut JNIEnv, context: &JObject, account: &str) -> Result<()> {
    let prefs = shared_prefs(env, context)?;
    let editor = env
        .call_method(
            &prefs,
            "edit",
            "()Landroid/content/SharedPreferences$Editor;",
            &[],
        )?
        .l()?;
    let k = env.new_string(account)?;
    env.call_method(
        &editor,
        "remove",
        "(Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;",
        &[JValue::Object(&k)],
    )?;
    env.call_method(&editor, "commit", "()Z", &[])?;
    Ok(())
}

fn cipher<'a>(env: &mut JNIEnv<'a>) -> Result<JObject<'a>> {
    let name = env.new_string(TRANSFORM)?;
    Ok(env
        .call_static_method(
            "javax/crypto/Cipher",
            "getInstance",
            "(Ljava/lang/String;)Ljavax/crypto/Cipher;",
            &[JValue::Object(&name)],
        )?
        .l()?)
}

fn do_final(env: &mut JNIEnv, cipher: &JObject, input: &[u8]) -> Result<Vec<u8>> {
    let arr = env.byte_array_from_slice(input)?;
    let out = env
        .call_method(cipher, "doFinal", "([B)[B", &[JValue::Object(&arr)])?
        .l()?;
    Ok(env.convert_byte_array(JByteArray::from(out))?)
}

/// 저장 포맷: `base64(iv).base64(ciphertext)`. GCM 이라 IV 는 비밀이 아니다.
fn encode(iv: &[u8], ct: &[u8]) -> String {
    format!("{}.{}", B64.encode(iv), B64.encode(ct))
}

fn decode(s: &str) -> Result<(Vec<u8>, Vec<u8>)> {
    let (iv_b64, ct_b64) = s
        .split_once('.')
        .ok_or_else(|| anyhow!("저장된 자격증명 형식이 손상되었습니다"))?;
    Ok((B64.decode(iv_b64)?, B64.decode(ct_b64)?))
}

pub fn set(account: &str, secret: &str) -> Result<()> {
    let plaintext = secret.as_bytes().to_vec();
    with_env(|env, context| {
        let key = master_key(env)?;
        let c = cipher(env)?;
        env.call_method(
            &c,
            "init",
            "(ILjava/security/Key;)V",
            &[JValue::Int(ENCRYPT_MODE), JValue::Object(&key)],
        )?;
        let iv_obj = env.call_method(&c, "getIV", "()[B", &[])?.l()?;
        let iv = env.convert_byte_array(JByteArray::from(iv_obj))?;
        let ct = do_final(env, &c, &plaintext)?;
        write_pref(env, context, account, &encode(&iv, &ct))
    })
}

pub fn get(account: &str) -> Result<String> {
    with_env(|env, context| {
        let stored =
            read_pref(env, context, account)?.ok_or_else(|| anyhow!("저장된 API 키가 없습니다"))?;
        let (iv, ct) = decode(&stored)?;

        let key = master_key(env)?;
        let c = cipher(env)?;
        let iv_arr = env.byte_array_from_slice(&iv)?;
        let spec = env.new_object(
            "javax/crypto/spec/GCMParameterSpec",
            "(I[B)V",
            &[JValue::Int(GCM_TAG_BITS), JValue::Object(&iv_arr)],
        )?;
        env.call_method(
            &c,
            "init",
            "(ILjava/security/Key;Ljava/security/spec/AlgorithmParameterSpec;)V",
            &[
                JValue::Int(DECRYPT_MODE),
                JValue::Object(&key),
                JValue::Object(&spec),
            ],
        )?;
        let plain = do_final(env, &c, &ct)?;
        String::from_utf8(plain).map_err(|_| anyhow!("복호화 결과가 UTF-8 이 아닙니다"))
    })
}

pub fn exists(account: &str) -> Result<bool> {
    with_env(|env, context| Ok(read_pref(env, context, account)?.is_some()))
}

pub fn delete(account: &str) -> Result<()> {
    // 이미 없어도 성공 (멱등).
    with_env(|env, context| remove_pref(env, context, account))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let (iv, ct) = (vec![1u8, 2, 3], vec![9u8, 8, 7, 6]);
        let (iv2, ct2) = decode(&encode(&iv, &ct)).unwrap();
        assert_eq!((iv, ct), (iv2, ct2));
    }

    #[test]
    fn decode_rejects_corrupt() {
        assert!(decode("구분자없음").is_err());
    }
}
