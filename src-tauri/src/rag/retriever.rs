// 검색 결과 → LLM 컨텍스트 조립 + 출처(Citation) 생성. 데스크탑판 이식.
use crate::rag::vectordb::Hit;
use crate::rag::Citation;

pub struct Context {
    pub prompt_context: String,
    pub citations: Vec<Citation>,
}

/// snippet 최대 길이(문자).
const SNIPPET_CHARS: usize = 120;

fn snippet(text: &str) -> String {
    let s: String = text.chars().take(SNIPPET_CHARS).collect();
    if text.chars().count() > SNIPPET_CHARS {
        format!("{s}…")
    } else {
        s
    }
}

/// 상위 hit 들을 문자 예산 내로 컨텍스트 조립. 각 조각에 `[파일 p.N]` 태그.
/// `filename_of`: doc_id → 파일명 해석 (DocStore 주입).
pub fn assemble(hits: &[Hit], filename_of: impl Fn(&str) -> String, max_chars: usize) -> Context {
    let mut prompt_context = String::new();
    let mut citations = Vec::new();

    for h in hits {
        let filename = filename_of(&h.doc_id);
        let tag = format!("[{} p.{}]\n{}\n\n", filename, h.page, h.text);
        if prompt_context.chars().count() + tag.chars().count() > max_chars
            && !prompt_context.is_empty()
        {
            break; // 예산 초과 → 중단 (최소 1개는 포함)
        }
        prompt_context.push_str(&tag);
        citations.push(Citation {
            doc_id: h.doc_id.clone(),
            filename,
            page: h.page,
            snippet: snippet(&h.text),
        });
    }

    Context {
        prompt_context,
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hit(doc: &str, page: u32, text: &str, score: f32) -> Hit {
        Hit {
            doc_id: doc.into(),
            chunk_index: 0,
            page,
            text: text.into(),
            score,
        }
    }

    #[test]
    fn assembles_with_tags_and_citations() {
        let hits = vec![hit("d1", 3, "비자 연장 서류", 0.9)];
        let ctx = assemble(&hits, |id| format!("{id}.pdf"), 1000);
        assert!(ctx.prompt_context.contains("[d1.pdf p.3]"));
        assert!(ctx.prompt_context.contains("비자 연장 서류"));
        assert_eq!(ctx.citations.len(), 1);
        assert_eq!(ctx.citations[0].page, 3);
    }

    #[test]
    fn respects_char_budget() {
        let hits = vec![
            hit("a", 1, &"x".repeat(80), 0.9),
            hit("b", 2, &"y".repeat(80), 0.8),
            hit("c", 3, &"z".repeat(80), 0.7),
        ];
        let ctx = assemble(&hits, |id| id.to_string(), 50);
        assert_eq!(ctx.citations.len(), 1); // 최소 1개 보장
        assert_eq!(ctx.citations[0].doc_id, "a");
    }

    #[test]
    fn snippet_truncates_long_text() {
        let long = "가".repeat(200);
        let ctx = assemble(&[hit("d", 1, &long, 0.9)], |_| "f".into(), 100000);
        assert!(ctx.citations[0].snippet.ends_with('…'));
    }

    #[test]
    fn empty_hits_yield_empty_context() {
        let ctx = assemble(&[], |_| "f".into(), 1000);
        assert!(ctx.prompt_context.is_empty());
        assert!(ctx.citations.is_empty());
    }
}
