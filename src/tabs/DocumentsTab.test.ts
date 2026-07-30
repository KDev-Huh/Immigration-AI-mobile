import { describe, expect, it } from "vitest";
import { displayName } from "./DocumentsTab";

describe("displayName", () => {
  it("URI 에 실제 파일명이 있으면 그대로 쓴다", () => {
    expect(
      displayName(
        "content://com.android.externalstorage.documents/document/primary%3ADownload%2Fanswer.pdf",
      ),
    ).toBe("answer.pdf");
    expect(displayName("/storage/emulated/0/Download/사증민원.pdf")).toBe(
      "사증민원.pdf",
    );
  });

  it("문서 ID 형태면 날짜 기반 이름으로 대체한다", () => {
    // 이 URI 가 예전에 "지원하지 않는 포맷: ." 오류를 냈다.
    const name = displayName(
      "content://com.android.providers.downloads.documents/document/msf%3A1000000123",
    );
    expect(name).toMatch(/^문서-\d{8}-\d{4}\.pdf$/);
  });

  it("깨진 인코딩이어도 예외를 던지지 않는다", () => {
    expect(() => displayName("content://x/%E0%A4%A")).not.toThrow();
  });
});
