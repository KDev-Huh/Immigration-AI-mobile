// 텍스트 추출. 현재 지원: PDF(텍스트). 페이지 경계 보존(출처 표시 필수).
// 데스크탑판 이식. 차이: 모바일 파일 피커가 content:// URI 를 주므로 경로가 아니라
// **바이트**를 입력으로 받는다.
use anyhow::{anyhow, Result};
use std::path::Path;

pub struct Page {
    pub number: u32, // 1-base (pdf-extract는 페이지 경계 미보존 → 내용 기준 근사)
    pub text: String,
}

/// 텍스트 없으면 스캔/추출불가로 간주하는 임계값.
const MIN_TEXT_CHARS: usize = 20;
/// 페이지 경계가 없어 내용 기준 pseudo-page 분할(~문자수). 출처 대략 위치용.
const CONTENT_CHARS_PER_PAGE: usize = 1500;

/// 파일명 확장자로 지원 포맷 판정.
pub fn detect_pdf(filename: &str) -> Result<()> {
    let ext = Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    if ext == "pdf" {
        Ok(())
    } else {
        Err(anyhow!("지원하지 않는 포맷: .{ext} (현재 PDF만 지원)"))
    }
}

/// PDF 바이트 → 페이지 배열. 내용 기준 pseudo-page.
pub fn parse_bytes(bytes: &[u8]) -> Result<Vec<Page>> {
    let raw = pdf_extract::extract_text_from_mem(bytes)
        .map_err(|e| anyhow!("PDF 텍스트 추출 실패: {e}"))?;
    if raw.trim().chars().count() < MIN_TEXT_CHARS {
        return Err(anyhow!(
            "추출된 텍스트가 거의 없습니다 (스캔 PDF일 수 있음, OCR 미지원)"
        ));
    }
    Ok(paginate(&raw, CONTENT_CHARS_PER_PAGE))
}

/// 저장된 원본 파일 경로 → 페이지 배열.
pub fn parse_file(path: &Path) -> Result<Vec<Page>> {
    let bytes = std::fs::read(path).map_err(|e| anyhow!("파일 읽기 실패: {e}"))?;
    parse_bytes(&bytes)
}

/// 문단(개행) 경계 유지하며 ~per 문자마다 페이지 증가.
fn paginate(text: &str, per: usize) -> Vec<Page> {
    let mut pages = Vec::new();
    let mut cur = String::new();
    let mut num = 1u32;
    for para in text.split('\n') {
        let para = para.trim();
        if para.is_empty() {
            continue;
        }
        if !cur.is_empty() && cur.chars().count() + para.chars().count() > per {
            pages.push(Page {
                number: num,
                text: std::mem::take(&mut cur),
            });
            num += 1;
        }
        if !cur.is_empty() {
            cur.push('\n');
        }
        cur.push_str(para);
    }
    if !cur.is_empty() {
        pages.push(Page {
            number: num,
            text: cur,
        });
    }
    pages
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_pdf_rejected() {
        assert!(detect_pdf("a.docx").is_err());
        assert!(detect_pdf("a.hwp").is_err());
        assert!(detect_pdf("a.txt").is_err());
        assert!(detect_pdf("a.pdf").is_ok());
        assert!(detect_pdf("A.PDF").is_ok()); // 대소문자 무시
    }

    #[test]
    fn garbage_bytes_error_not_panic() {
        assert!(parse_bytes(b"not a pdf at all").is_err());
    }

    #[test]
    fn paginate_splits_by_content() {
        let paras: Vec<String> = (0..5).map(|_| "가".repeat(100)).collect();
        let text = paras.join("\n");
        let pages = paginate(&text, 250);
        assert!(pages.len() >= 2, "pages={}", pages.len());
        assert_eq!(pages[0].number, 1);
        assert_eq!(pages[1].number, 2);
        let total: usize = pages
            .iter()
            .map(|p| p.text.chars().filter(|c| *c == '가').count())
            .sum();
        assert_eq!(total, 500); // 손실 없음
    }

    #[test]
    fn paginate_single_page_for_short() {
        let pages = paginate("짧은 문서", 1500);
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].number, 1);
    }
}
