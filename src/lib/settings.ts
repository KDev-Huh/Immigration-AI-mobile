// 앱 설정 (공급자·모델·표시) 로컬 영속. API 키는 여기 절대 저장 금지 — 보안저장(Rust)만.
import type { CloudProvider } from "@/types";

export const PROVIDERS: { id: CloudProvider; label: string }[] = [
  { id: "openai", label: "OpenAI (ChatGPT)" },
  { id: "anthropic", label: "Anthropic (Claude)" },
  { id: "gemini", label: "Google Gemini" },
];

/** provider별 추천 모델. 직접 입력도 허용. */
export const MODELS: Record<CloudProvider, string[]> = {
  openai: ["gpt-4o", "gpt-4o-mini", "gpt-4.1"],
  anthropic: ["claude-sonnet-5", "claude-opus-5", "claude-haiku-4-5-20251001"],
  // 모델 ID 는 Google 공식 목록과 일치해야 한다 — 존재하지 않는 ID 를 고르면 404 로 실패한다.
  gemini: ["gemini-3.6-flash", "gemini-3.5-flash-lite", "gemini-2.5-pro"],
};

const PROVIDER_KEY = "cloud-provider";
const FONT_SIZE_KEY = "font-size";
const modelKey = (p: CloudProvider) => `cloud-model-${p}`;

export type FontSize = "small" | "normal" | "large" | "xlarge";

export const FONT_SIZES: { id: FontSize; label: string; px: number }[] = [
  { id: "small", label: "작게", px: 15 },
  { id: "normal", label: "보통", px: 16 },
  { id: "large", label: "크게", px: 18 },
  { id: "xlarge", label: "아주 크게", px: 20 },
];

export const defaultModel = (p: CloudProvider): string => MODELS[p][0];

/** 저장값이 유효한 provider 인지 판정 (순수). */
export const isProvider = (v: unknown): v is CloudProvider =>
  v === "openai" || v === "anthropic" || v === "gemini";

export const isFontSize = (v: unknown): v is FontSize =>
  v === "small" || v === "normal" || v === "large" || v === "xlarge";

export function getProvider(): CloudProvider {
  const v = localStorage.getItem(PROVIDER_KEY);
  return isProvider(v) ? v : "openai";
}

export function setProvider(p: CloudProvider) {
  localStorage.setItem(PROVIDER_KEY, p);
}

export function getModel(p: CloudProvider): string {
  return localStorage.getItem(modelKey(p)) || defaultModel(p);
}

export function setModel(p: CloudProvider, m: string) {
  localStorage.setItem(modelKey(p), m);
}

export function getFontSize(): FontSize {
  const v = localStorage.getItem(FONT_SIZE_KEY);
  return isFontSize(v) ? v : "normal";
}

export function setFontSize(size: FontSize) {
  localStorage.setItem(FONT_SIZE_KEY, size);
  applyFontSize(size);
}

export function applyFontSize(size = getFontSize()) {
  const px = FONT_SIZES.find((s) => s.id === size)?.px ?? 16;
  document.documentElement.style.setProperty("--app-font-size", `${px}px`);
}
