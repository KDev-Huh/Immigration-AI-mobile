// 클라우드 LLM/임베딩 HTTP 클라이언트. 키는 security 모듈(보안저장)에서 조회해 주입.
// 모바일판은 생성뿐 아니라 **임베딩도 클라우드**(OpenAI)를 쓴다 — 로컬 임베딩 없음.
use crate::security::CloudProvider;
use anyhow::{anyhow, Result};
use serde_json::Value;

pub const OPENAI_CHAT_URL: &str = "https://api.openai.com/v1/chat/completions";
pub const OPENAI_EMBED_URL: &str = "https://api.openai.com/v1/embeddings";
pub const ANTHROPIC_URL: &str = "https://api.anthropic.com/v1/messages";
pub const GEMINI_URL_BASE: &str = "https://generativelanguage.googleapis.com/v1beta/models";

pub const OPENAI_MODEL: &str = "gpt-4o";
pub const ANTHROPIC_MODEL: &str = "claude-sonnet-5";
pub const GEMINI_MODEL: &str = "gemini-3.6-flash";
pub const ANTHROPIC_VERSION: &str = "2023-06-01";
const MAX_TOKENS: u32 = 1024;

fn client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| anyhow!("HTTP 클라이언트 생성 실패: {e}"))
}

// --- OpenAI 생성 ---
fn openai_body(model: &str, system: &str, user: &str) -> Value {
    serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": system },
            { "role": "user", "content": user },
        ],
    })
}

fn parse_openai(body: &str) -> Result<String> {
    let v: Value = serde_json::from_str(body).map_err(|e| anyhow!("OpenAI 응답 파싱 실패: {e}"))?;
    v["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("OpenAI 응답 형식 오류: {body}"))
}

// --- Anthropic 생성 ---
fn anthropic_body(model: &str, system: &str, user: &str) -> Value {
    serde_json::json!({
        "model": model,
        "max_tokens": MAX_TOKENS,
        "system": system,
        "messages": [ { "role": "user", "content": user } ],
    })
}

fn parse_anthropic(body: &str) -> Result<String> {
    let v: Value =
        serde_json::from_str(body).map_err(|e| anyhow!("Anthropic 응답 파싱 실패: {e}"))?;
    v["content"][0]["text"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("Anthropic 응답 형식 오류: {body}"))
}

// --- Gemini 생성 ---
fn gemini_url(model: &str) -> String {
    let model = model.strip_prefix("models/").unwrap_or(model);
    format!("{GEMINI_URL_BASE}/{model}:generateContent")
}

fn gemini_body(system: &str, user: &str) -> Value {
    serde_json::json!({
        "system_instruction": {
            "parts": [{ "text": system }],
        },
        "contents": [
            {
                "role": "user",
                "parts": [{ "text": user }],
            },
        ],
        "generationConfig": {
            "maxOutputTokens": MAX_TOKENS,
        },
    })
}

fn parse_gemini(body: &str) -> Result<String> {
    let v: Value = serde_json::from_str(body).map_err(|e| anyhow!("Gemini 응답 파싱 실패: {e}"))?;
    let parts = v["candidates"][0]["content"]["parts"]
        .as_array()
        .ok_or_else(|| anyhow!("Gemini 응답 형식 오류: {body}"))?;
    let text = parts
        .iter()
        .filter_map(|p| p["text"].as_str())
        .collect::<Vec<_>>()
        .join("");
    if text.is_empty() {
        return Err(anyhow!("Gemini 응답에 텍스트가 없습니다: {body}"));
    }
    Ok(text)
}

/// 클라우드 생성. model 미지정 시 provider 기본 모델.
pub async fn generate(
    provider: CloudProvider,
    api_key: &str,
    system: &str,
    user: &str,
    model: Option<&str>,
) -> Result<String> {
    let client = client()?;
    match provider {
        CloudProvider::Openai => {
            let m = model.unwrap_or(OPENAI_MODEL);
            let resp = client
                .post(OPENAI_CHAT_URL)
                .bearer_auth(api_key)
                .json(&openai_body(m, system, user))
                .send()
                .await
                .map_err(|e| anyhow!("OpenAI 연결 실패: {e}"))?;
            let resp = check_status("OpenAI", resp).await?;
            parse_openai(&resp)
        }
        CloudProvider::Anthropic => {
            let m = model.unwrap_or(ANTHROPIC_MODEL);
            let resp = client
                .post(ANTHROPIC_URL)
                .header("x-api-key", api_key)
                .header("anthropic-version", ANTHROPIC_VERSION)
                .json(&anthropic_body(m, system, user))
                .send()
                .await
                .map_err(|e| anyhow!("Anthropic 연결 실패: {e}"))?;
            let resp = check_status("Anthropic", resp).await?;
            parse_anthropic(&resp)
        }
        CloudProvider::Gemini => {
            let m = model.unwrap_or(GEMINI_MODEL);
            let resp = client
                .post(gemini_url(m))
                .header("x-goog-api-key", api_key)
                .json(&gemini_body(system, user))
                .send()
                .await
                .map_err(|e| anyhow!("Gemini 연결 실패: {e}"))?;
            let resp = check_status("Gemini", resp).await?;
            parse_gemini(&resp)
        }
    }
}

// --- OpenAI 임베딩 ---
fn embed_body(model: &str, texts: &[String]) -> Value {
    serde_json::json!({ "model": model, "input": texts })
}

/// 임베딩 응답 파싱. `index` 기준 정렬로 **입력 순서를 보장**한다.
/// 순서가 어긋나면 청크↔벡터 대응이 깨져 출처가 틀리므로 여기서 못박는다.
fn parse_embeddings(body: &str, expected: usize) -> Result<Vec<Vec<f32>>> {
    let v: Value = serde_json::from_str(body).map_err(|e| anyhow!("임베딩 응답 파싱 실패: {e}"))?;
    let data = v["data"]
        .as_array()
        .ok_or_else(|| anyhow!("임베딩 응답 형식 오류: {body}"))?;
    if data.len() != expected {
        return Err(anyhow!(
            "임베딩 개수 불일치: 요청 {expected}, 응답 {}",
            data.len()
        ));
    }

    let mut indexed: Vec<(usize, Vec<f32>)> = Vec::with_capacity(data.len());
    for item in data {
        let idx = item["index"]
            .as_u64()
            .ok_or_else(|| anyhow!("임베딩 index 누락"))? as usize;
        let vec = item["embedding"]
            .as_array()
            .ok_or_else(|| anyhow!("임베딩 벡터 누락"))?
            .iter()
            .map(|n| {
                n.as_f64()
                    .map(|f| f as f32)
                    .ok_or_else(|| anyhow!("임베딩 값 오류"))
            })
            .collect::<Result<Vec<f32>>>()?;
        indexed.push((idx, vec));
    }
    indexed.sort_by_key(|(i, _)| *i);
    Ok(indexed.into_iter().map(|(_, v)| v).collect())
}

/// OpenAI 임베딩 1배치. 임베딩은 **항상 OpenAI** — Anthropic 은 임베딩 API가 없다.
pub async fn embed(api_key: &str, model: &str, texts: &[String]) -> Result<Vec<Vec<f32>>> {
    if texts.is_empty() {
        return Ok(Vec::new());
    }
    let resp = client()?
        .post(OPENAI_EMBED_URL)
        .bearer_auth(api_key)
        .json(&embed_body(model, texts))
        .send()
        .await
        .map_err(|e| anyhow!("OpenAI 임베딩 연결 실패: {e}"))?;
    let body = check_status("OpenAI 임베딩", resp).await?;
    parse_embeddings(&body, texts.len())
}

/// 상태 코드 확인 + 본문 반환. 실패 시 API 가 준 에러 메시지를 그대로 보여준다.
async fn check_status(who: &str, resp: reqwest::Response) -> Result<String> {
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    if status.is_success() {
        return Ok(body);
    }
    let detail = serde_json::from_str::<Value>(&body)
        .ok()
        .and_then(|v| v["error"]["message"].as_str().map(|s| s.to_string()))
        .unwrap_or(body);
    Err(anyhow!("{who} 오류 ({status}): {detail}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openai_body_shape() {
        let b = openai_body("gpt-4o", "sys", "usr");
        assert_eq!(b["model"], "gpt-4o");
        assert_eq!(b["messages"][0]["role"], "system");
        assert_eq!(b["messages"][1]["content"], "usr");
    }

    #[test]
    fn parse_openai_ok_and_err() {
        let body = r#"{"choices":[{"message":{"content":"필요 서류 안내"}}]}"#;
        assert_eq!(parse_openai(body).unwrap(), "필요 서류 안내");
        assert!(parse_openai(r#"{"error":"x"}"#).is_err());
    }

    #[test]
    fn anthropic_body_shape() {
        let b = anthropic_body("claude-sonnet-5", "sys", "usr");
        assert_eq!(b["model"], "claude-sonnet-5");
        assert_eq!(b["system"], "sys");
        assert_eq!(b["max_tokens"], MAX_TOKENS);
    }

    #[test]
    fn parse_anthropic_ok_and_err() {
        let body = r#"{"content":[{"type":"text","text":"연장 절차"}]}"#;
        assert_eq!(parse_anthropic(body).unwrap(), "연장 절차");
        assert!(parse_anthropic(r#"{"content":[]}"#).is_err());
    }

    #[test]
    fn gemini_body_shape() {
        let b = gemini_body("sys", "usr");
        assert_eq!(b["system_instruction"]["parts"][0]["text"], "sys");
        assert_eq!(b["contents"][0]["role"], "user");
        assert_eq!(b["contents"][0]["parts"][0]["text"], "usr");
        assert_eq!(b["generationConfig"]["maxOutputTokens"], MAX_TOKENS);
    }

    #[test]
    fn parse_gemini_ok_and_err() {
        let body = r#"{"candidates":[{"content":{"parts":[{"text":"필요 "},{"text":"서류"}]}}]}"#;
        assert_eq!(parse_gemini(body).unwrap(), "필요 서류");
        assert!(parse_gemini(r#"{"candidates":[]}"#).is_err());
    }

    #[test]
    fn gemini_url_accepts_plain_or_prefixed_model() {
        assert_eq!(
            gemini_url("gemini-3.6-flash"),
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-3.6-flash:generateContent"
        );
        assert_eq!(
            gemini_url("models/gemini-3.6-flash"),
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-3.6-flash:generateContent"
        );
    }

    #[test]
    fn embeddings_reordered_by_index() {
        // API 가 순서를 뒤집어 줘도 index 기준으로 복원돼야 한다.
        let body = r#"{"data":[
            {"index":1,"embedding":[0.0,1.0]},
            {"index":0,"embedding":[1.0,0.0]}
        ]}"#;
        let v = parse_embeddings(body, 2).unwrap();
        assert_eq!(v[0], vec![1.0, 0.0]);
        assert_eq!(v[1], vec![0.0, 1.0]);
    }

    #[test]
    fn embeddings_count_mismatch_is_error() {
        let body = r#"{"data":[{"index":0,"embedding":[1.0]}]}"#;
        assert!(parse_embeddings(body, 2).is_err());
    }

    #[test]
    fn embed_body_shape() {
        let b = embed_body("text-embedding-3-small", &["a".into(), "b".into()]);
        assert_eq!(b["model"], "text-embedding-3-small");
        assert_eq!(b["input"][1], "b");
    }
}
