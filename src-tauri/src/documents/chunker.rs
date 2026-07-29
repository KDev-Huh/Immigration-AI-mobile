// 페이지 → 청크. 페이지 경계 존중(청크는 한 페이지 내) + 슬라이딩 오버랩.
// 데스크탑판 이식. 토크나이저 대신 문자 윈도우 근사.
use crate::documents::parser::Page;
use crate::documents::Chunk;

/// 토큰↔문자 근사 계수 (KR/EN 혼합 보수값).
pub const CHARS_PER_TOKEN: usize = 3;

pub struct ChunkConfig {
    pub target_tokens: usize,
    pub overlap_tokens: usize,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            target_tokens: 500,
            overlap_tokens: 64,
        }
    }
}

impl ChunkConfig {
    fn target_chars(&self) -> usize {
        (self.target_tokens * CHARS_PER_TOKEN).max(1)
    }
    fn overlap_chars(&self) -> usize {
        // 진행 보장: overlap < target
        (self.overlap_tokens * CHARS_PER_TOKEN).min(self.target_chars().saturating_sub(1))
    }
}

/// 토큰 수 근사치.
pub fn estimate_tokens(text: &str) -> usize {
    text.chars().count().div_ceil(CHARS_PER_TOKEN)
}

/// 페이지들 → 청크들. index 는 문서 전체 통산(0-base), page 메타 보존.
pub fn chunk(doc_id: &str, pages: &[Page], cfg: &ChunkConfig) -> Vec<Chunk> {
    let target = cfg.target_chars();
    let overlap = cfg.overlap_chars();
    let mut out = Vec::new();
    let mut index = 0usize;

    for page in pages {
        let chars: Vec<char> = page.text.chars().collect();
        if chars.is_empty() {
            continue;
        }
        for (s, e) in windows(&chars, target, overlap) {
            let text: String = chars[s..e].iter().collect();
            out.push(Chunk {
                doc_id: doc_id.to_string(),
                index,
                text,
                page: page.number,
            });
            index += 1;
        }
    }
    out
}

/// [start,end) 윈도우 나열. 소프트 경계(끝 부근 공백 선호), 오버랩 스텝.
fn windows(chars: &[char], target: usize, overlap: usize) -> Vec<(usize, usize)> {
    let len = chars.len();
    let mut spans = Vec::new();
    let mut start = 0usize;
    let lookback = (target / 5).max(1);

    while start < len {
        let hard_end = (start + target).min(len);
        let mut end = hard_end;
        if hard_end < len {
            let floor = hard_end.saturating_sub(lookback).max(start + 1);
            for i in (floor..hard_end).rev() {
                if chars[i - 1].is_whitespace() {
                    end = i;
                    break;
                }
            }
        }
        spans.push((start, end));
        if end >= len {
            break;
        }
        let next = end.saturating_sub(overlap);
        start = if next <= start { end } else { next };
    }
    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(number: u32, text: &str) -> Page {
        Page {
            number,
            text: text.to_string(),
        }
    }

    fn cfg(target_tokens: usize, overlap_tokens: usize) -> ChunkConfig {
        ChunkConfig {
            target_tokens,
            overlap_tokens,
        }
    }

    /// 공백 없는 시퀀스 — 하드 컷/오버랩 정확 검증용.
    fn digits(n: usize) -> String {
        (0..n).map(|i| char::from(b'0' + (i % 10) as u8)).collect()
    }

    #[test]
    fn long_page_splits_keeps_page_number() {
        let chunks = chunk("d", &[page(7, &digits(75))], &cfg(10, 0));
        assert!(chunks.len() >= 3);
        assert!(chunks.iter().all(|k| k.page == 7));
        assert_eq!(chunks[0].index, 0);
        assert_eq!(chunks[1].index, 1);
    }

    #[test]
    fn no_overlap_reconstructs_original() {
        let original = digits(95);
        let chunks = chunk("d", &[page(1, &original)], &cfg(10, 0));
        let joined: String = chunks.iter().map(|k| k.text.clone()).collect();
        assert_eq!(joined, original); // 손실 없음
    }

    #[test]
    fn overlap_shares_boundary_text() {
        let chunks = chunk("d", &[page(1, &digits(80))], &cfg(10, 4)); // target 30, overlap 12
        assert!(chunks.len() >= 2);
        let tail: String = chunks[0]
            .text
            .chars()
            .rev()
            .take(12)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        let head: String = chunks[1].text.chars().take(12).collect();
        assert_eq!(tail, head);
    }

    #[test]
    fn multi_page_maps_page_numbers() {
        let chunks = chunk("d", &[page(1, "first"), page(2, "second")], &cfg(100, 0));
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].page, 1);
        assert_eq!(chunks[1].page, 2);
    }

    #[test]
    fn empty_page_skipped() {
        let chunks = chunk("d", &[page(1, ""), page(2, "text")], &cfg(100, 0));
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].page, 2);
    }

    #[test]
    fn soft_boundary_prefers_whitespace() {
        let text = format!("{} {}", "a".repeat(28), "b".repeat(40));
        let chunks = chunk("d", &[page(1, &text)], &cfg(10, 0));
        assert!(!chunks[0].text.contains('b'), "got: {:?}", chunks[0].text);
    }

    #[test]
    fn estimate_tokens_rounds_up() {
        assert_eq!(estimate_tokens("abcd"), 2); // 4/3 → 2
        assert_eq!(estimate_tokens(""), 0);
    }
}
