// 앱 설정 (공급자·모델) 로컬 영속. API 키는 여기 절대 저장 금지 — 보안저장(Rust)만.
import type { CloudProvider } from "@/types";

export const PROVIDERS: { id: CloudProvider; label: string }[] = [
  { id: "openai", label: "OpenAI (ChatGPT)" },
  { id: "anthropic", label: "Anthropic (Claude)" },
];

/** provider별 추천 모델. 직접 입력도 허용. */
export const MODELS: Record<CloudProvider, string[]> = {
  openai: ["gpt-4o", "gpt-4o-mini", "gpt-4.1"],
  anthropic: ["claude-sonnet-5", "claude-opus-5", "claude-haiku-4-5-20251001"],
};

const PROVIDER_KEY = "cloud-provider";
const modelKey = (p: CloudProvider) => `cloud-model-${p}`;

export const defaultModel = (p: CloudProvider): string => MODELS[p][0];

/** 저장값이 유효한 provider 인지 판정 (순수). */
export const isProvider = (v: unknown): v is CloudProvider =>
  v === "openai" || v === "anthropic";

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
