// 유출 태그 표시/판정 헬퍼 — 테스트 가능한 순수 로직.
import type { Sensitivity } from "@/types";

export const sensitivityLabel = (s: Sensitivity): string =>
  s === "confidential" ? "유출불가" : "유출가능";

/**
 * 모바일 업로드 허용 여부.
 * 모바일은 클라우드 전송이 전제이므로 유출불가 문서는 **절대** 업로드 불가.
 * (Rust 쪽에서도 같은 판정을 하며, 그쪽이 최후 방어선이다.)
 */
export const allowsUpload = (s: Sensitivity): boolean => s === "leakable";

/** 업로드 거부 사유 문구. */
export const uploadRejectReason =
  "유출불가 문서는 모바일에서 업로드할 수 없습니다 (클라우드 전송 전제). 데스크탑판을 사용하세요.";
