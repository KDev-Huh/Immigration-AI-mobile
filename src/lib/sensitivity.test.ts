import { describe, expect, it } from "vitest";
import { allowsUpload, sensitivityLabel } from "./sensitivity";

describe("sensitivity", () => {
  it("유출불가 문서는 모바일 업로드 불허 (보안 경계)", () => {
    expect(allowsUpload("confidential")).toBe(false);
    expect(allowsUpload("leakable")).toBe(true);
  });

  it("라벨 매핑", () => {
    expect(sensitivityLabel("confidential")).toBe("유출불가");
    expect(sensitivityLabel("leakable")).toBe("유출가능");
  });
});
