// 탭 B — 채팅. 검색 대상은 항상 유출가능 문서 단일 컬렉션(Rust 강제).
// 공급자/모델/키는 설정 탭에서 관리하고 여기서는 읽기만 한다.
import { useEffect, useState } from "react";
import { ask, hasApiKey } from "@/lib/ipc";
import { getModel, getProvider } from "@/lib/settings";
import type { ChatAnswer } from "@/types";
import ChatPane from "@/components/ChatPane";

export default function ChatTab() {
  const [provider, setProviderState] = useState(getProvider());
  const [model, setModelState] = useState(() => getModel(getProvider()));
  const [ready, setReady] = useState<boolean | null>(null);

  // 설정 탭에서 바꿔도 반영되도록 탭 표시 때마다 재확인.
  useEffect(() => {
    const sync = () => {
      const p = getProvider();
      setProviderState(p);
      setModelState(getModel(p));
      // 임베딩은 항상 OpenAI → 채팅 공급자와 무관하게 OpenAI 키가 필요.
      Promise.all([hasApiKey("openai"), hasApiKey(p)])
        .then(([openai, chat]) => setReady(openai && chat))
        .catch(() => setReady(false));
    };
    sync();
    document.addEventListener("visibilitychange", sync);
    window.addEventListener("focus", sync);
    return () => {
      document.removeEventListener("visibilitychange", sync);
      window.removeEventListener("focus", sync);
    };
  }, []);

  const onAsk = (q: string): Promise<ChatAnswer> => ask(q, provider, model);

  const subtitle =
    ready === false
      ? "⚠️ API 키가 없습니다 — 설정 탭에서 등록하세요 (임베딩용 OpenAI 키 필수)"
      : `${provider} · ${model}`;

  return <ChatPane ns="cloud" onAsk={onAsk} subtitle={subtitle} />;
}
