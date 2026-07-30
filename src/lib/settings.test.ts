import { describe, expect, it } from "vitest";
import { defaultModel, FONT_SIZES, isFontSize, isProvider, MODELS } from "./settings";

describe("settings", () => {
  it("알 수 없는 저장값은 provider 로 인정하지 않음", () => {
    expect(isProvider("openai")).toBe(true);
    expect(isProvider("anthropic")).toBe(true);
    expect(isProvider("gemini")).toBe(true);
    expect(isProvider("browser")).toBe(false); // 브라우저 로그인 미지원
    expect(isProvider(null)).toBe(false);
  });

  it("공급자별 기본 모델이 존재", () => {
    expect(MODELS.openai.length).toBeGreaterThan(0);
    expect(MODELS.anthropic.length).toBeGreaterThan(0);
    expect(MODELS.gemini.length).toBeGreaterThan(0);
    expect(defaultModel("openai")).toBe(MODELS.openai[0]);
  });

  it("알 수 없는 글자 크기 저장값은 인정하지 않음", () => {
    expect(isFontSize("normal")).toBe(true);
    expect(isFontSize("xlarge")).toBe(true);
    expect(isFontSize("huge")).toBe(false);
    expect(FONT_SIZES.map((s) => s.id)).toContain("large");
  });
});
