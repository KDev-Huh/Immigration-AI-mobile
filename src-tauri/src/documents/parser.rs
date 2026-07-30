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

/// PDF 파일 시그니처. 규격상 파일 선두에 온다.
const PDF_MAGIC: &[u8] = b"%PDF-";
/// 시그니처를 찾을 선두 범위. 일부 파일은 BOM·공백이 앞에 붙는다.
const MAGIC_SCAN_BYTES: usize = 1024;

/// **내용**으로 PDF 판정.
///
/// 확장자로 판정하면 안 된다. Android 파일 피커는 `content://` URI 를 돌려주는데
/// 제공자에 따라 마지막 경로 조각이 파일명이 아니라 문서 ID(`msf:1000000123`)다.
/// 그러면 확장자가 아예 없어서 멀쩡한 PDF 가 "지원하지 않는 포맷"으로 거부된다.
/// 어차피 업로드 시점에 바이트를 갖고 있으므로 시그니처로 판정하는 쪽이 정확하다.
pub fn detect_pdf_bytes(bytes: &[u8]) -> Result<()> {
    let head = &bytes[..bytes.len().min(MAGIC_SCAN_BYTES)];
    if head.windows(PDF_MAGIC.len()).any(|w| w == PDF_MAGIC) {
        Ok(())
    } else {
        Err(anyhow!(
            "PDF 파일이 아닙니다 (현재 텍스트 PDF만 지원합니다)"
        ))
    }
}

/// 표시용 파일명 정리. 출처 표시(`[파일명 p.N]`)에 쓰이므로 경로·URI 조각을 걷어낸다.
/// 프론트가 이미 사람이 읽을 이름을 만들어 보내지만, 여기가 최종 방어선이다.
pub fn sanitize_filename(raw: &str) -> String {
    let base = raw.rsplit(['/', '\\']).next().unwrap_or(raw);
    let base: String = base.chars().filter(|c| !c.is_control()).collect();
    let base = base.trim();
    if base.is_empty() {
        return "문서.pdf".to_string();
    }
    if base.to_ascii_lowercase().ends_with(".pdf") {
        base.to_string()
    } else {
        format!("{base}.pdf")
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
    fn pdf_detected_by_content_not_extension() {
        // Android content:// URI 는 확장자가 없는 이름을 주기도 한다.
        // 판정은 오직 내용으로 한다.
        assert!(detect_pdf_bytes(b"%PDF-1.7\n...").is_ok());
        assert!(detect_pdf_bytes(b"\xef\xbb\xbf%PDF-1.4").is_ok()); // 앞에 BOM
        assert!(detect_pdf_bytes(b"PK\x03\x04 docx").is_err());
        assert!(detect_pdf_bytes(b"").is_err());
    }

    #[test]
    fn magic_not_searched_past_head() {
        // 본문 한참 뒤에 "%PDF-" 문자열이 있다고 PDF 로 인정하면 안 된다.
        let mut bytes = vec![b'x'; MAGIC_SCAN_BYTES + 64];
        bytes.extend_from_slice(b"%PDF-1.7");
        assert!(detect_pdf_bytes(&bytes).is_err());
    }

    #[test]
    fn filename_sanitized_for_citation() {
        assert_eq!(sanitize_filename("answer.pdf"), "answer.pdf");
        // URI 경로 조각 제거
        assert_eq!(
            sanitize_filename("primary:Download/사증민원.pdf"),
            "사증민원.pdf"
        );
        // 확장자 없는 이름에는 붙여준다
        assert_eq!(sanitize_filename("msf:1000000123"), "msf:1000000123.pdf");
        assert_eq!(sanitize_filename("   "), "문서.pdf");
        assert_eq!(sanitize_filename(""), "문서.pdf");
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
