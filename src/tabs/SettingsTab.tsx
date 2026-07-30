// 탭 C — 설정: 공급자/모델 선택 + API 키 등록·삭제.
// 키는 화면에 되읽지 않는다(보안저장에서 존재 여부만 조회).
import { useEffect, useState } from "react";
import { deleteApiKey, hasApiKey, setApiKey } from "@/lib/ipc";
import {
  FONT_SIZES,
  MODELS,
  PROVIDERS,
  type FontSize,
  getModel,
  getFontSize,
  getProvider,
  setFontSize as persistFontSize,
  setModel as persistModel,
  setProvider as persistProvider,
} from "@/lib/settings";
import type { CloudProvider } from "@/types";

export default function SettingsTab() {
  const [provider, setProvider] = useState<CloudProvider>(getProvider());
  const [model, setModel] = useState(() => getModel(getProvider()));
  const [keys, setKeys] = useState<Record<string, boolean>>({});
  const [input, setInput] = useState("");
  const [target, setTarget] = useState<CloudProvider>("openai");
  const [msg, setMsg] = useState<string | null>(null);
  const [fontSize, setFontSize] = useState<FontSize>(getFontSize());
  const keyPlaceholder = target === "gemini" ? "AIza..." : "sk-...";

  const refreshKeys = async () => {
    const entries = await Promise.all(
      PROVIDERS.map(async (p) => [p.id, await hasApiKey(p.id).catch(() => false)] as const),
    );
    setKeys(Object.fromEntries(entries));
  };

  useEffect(() => {
    refreshKeys();
  }, []);

  const onPickProvider = (p: CloudProvider) => {
    setProvider(p);
    persistProvider(p);
    const m = getModel(p);
    setModel(m);
  };

  const onPickModel = (m: string) => {
    setModel(m);
    persistModel(provider, m);
  };

  const onPickFontSize = (size: FontSize) => {
    setFontSize(size);
    persistFontSize(size);
  };

  const onSaveKey = async () => {
    const k = input.trim();
    if (!k) return;
    try {
      await setApiKey(target, k);
      setInput("");
      setMsg(`${target} 키 저장됨 (기기 보안저장)`);
      await refreshKeys();
    } catch (e) {
      setMsg(`키 저장 실패: ${e}`);
    }
  };

  const onDeleteKey = async (p: CloudProvider) => {
    try {
      await deleteApiKey(p);
      setMsg(`${p} 키 삭제됨`);
      await refreshKeys();
    } catch (e) {
      setMsg(`키 삭제 실패: ${e}`);
    }
  };

  return (
    <section className="tab-section">
      <header className="tab-header">
        <h2>설정</h2>
      </header>

      <div className="field">
        <label htmlFor="provider">채팅 공급자</label>
        <select
          id="provider"
          value={provider}
          onChange={(e) => onPickProvider(e.target.value as CloudProvider)}
        >
          {PROVIDERS.map((p) => (
            <option key={p.id} value={p.id}>
              {p.label}
            </option>
          ))}
        </select>
      </div>

      <div className="field">
        <label htmlFor="model">모델</label>
        <input
          id="model"
          list="model-options"
          value={model}
          onChange={(e) => onPickModel(e.target.value)}
        />
        <datalist id="model-options">
          {MODELS[provider].map((m) => (
            <option key={m} value={m} />
          ))}
        </datalist>
      </div>

      <h3>화면</h3>
      <div className="field">
        <span className="field-label">글자 크기</span>
        <div className="segmented" role="group" aria-label="글자 크기">
          {FONT_SIZES.map((s) => (
            <button
              key={s.id}
              type="button"
              className={fontSize === s.id ? "active" : ""}
              onClick={() => onPickFontSize(s.id)}
            >
              {s.label}
            </button>
          ))}
        </div>
      </div>

      <h3>API 키</h3>
      <p className="note">
        임베딩(문서 색인·검색)은 항상 OpenAI 를 사용합니다. Anthropic 또는 Gemini 로
        채팅하더라도 <b>OpenAI 키는 반드시 필요</b>합니다.
      </p>

      <ul className="key-list">
        {PROVIDERS.map((p) => (
          <li key={p.id}>
            <span>{p.label}</span>
            <span className={keys[p.id] ? "ok" : "missing"}>
              {keys[p.id] ? "저장됨 ✓" : "없음"}
            </span>
            {keys[p.id] && (
              <button className="danger" onClick={() => onDeleteKey(p.id)}>
                삭제
              </button>
            )}
          </li>
        ))}
      </ul>

      <div className="field">
        <label htmlFor="key-target">키 등록 대상</label>
        <select
          id="key-target"
          value={target}
          onChange={(e) => setTarget(e.target.value as CloudProvider)}
        >
          {PROVIDERS.map((p) => (
            <option key={p.id} value={p.id}>
              {p.label}
            </option>
          ))}
        </select>
      </div>

      <div className="field">
        <label htmlFor="key">API 키</label>
        <input
          id="key"
          type="password"
          autoComplete="off"
          autoCorrect="off"
          autoCapitalize="none"
          spellCheck={false}
          placeholder={keyPlaceholder}
          value={input}
          onChange={(e) => setInput(e.target.value)}
        />
        <button className="primary" onClick={onSaveKey} disabled={!input.trim()}>
          저장
        </button>
      </div>

      {msg && <p className="note">{msg}</p>}

      <h3>보안</h3>
      <ul className="note">
        <li>키는 iOS Keychain / Android Keystore 에 저장됩니다 (평문 파일 저장 없음).</li>
        <li>문서·벡터는 기기 로컬에만 저장됩니다. 원격 서버·텔레메트리 없음.</li>
        <li>유출불가 문서는 업로드할 수 없습니다 — 데스크탑판을 사용하세요.</li>
      </ul>
    </section>
  );
}
