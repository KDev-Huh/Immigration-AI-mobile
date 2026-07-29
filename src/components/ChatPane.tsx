// 모바일 채팅 UI. 세션 목록은 상시 사이드바가 아니라 오버레이 드로어(좁은 화면 대응).
// 세션은 localStorage 영속. 답변 하단에 출처(파일명·페이지) 표시 — 컨벤션상 필수.
import { useEffect, useMemo, useRef, useState } from "react";
import type { ChatAnswer, ChatSession, Message } from "@/types";

interface Props {
  ns: string;
  onAsk: (query: string) => Promise<ChatAnswer>;
  /** 헤더 우측에 표시할 현재 설정 요약 등 */
  subtitle?: string;
}

const uid = () => Math.random().toString(36).slice(2) + Date.now().toString(36);
const keyOf = (ns: string) => `chat-sessions-${ns}`;

function load(ns: string): ChatSession[] {
  try {
    const raw = JSON.parse(localStorage.getItem(keyOf(ns)) || "[]");
    return Array.isArray(raw) ? raw : [];
  } catch {
    return [];
  }
}

function save(ns: string, sessions: ChatSession[]) {
  try {
    localStorage.setItem(keyOf(ns), JSON.stringify(sessions));
  } catch {
    // 저장 실패(용량 초과 등)는 채팅 자체를 막지 않는다.
  }
}

export default function ChatPane({ ns, onAsk, subtitle }: Props) {
  const [sessions, setSessions] = useState<ChatSession[]>(() => load(ns));
  const [activeId, setActiveId] = useState<string | null>(
    () => load(ns)[0]?.id ?? null,
  );
  const [input, setInput] = useState("");
  const [busy, setBusy] = useState(false);
  const [drawer, setDrawer] = useState(false);
  const scrollRef = useRef<HTMLDivElement>(null);

  const active = useMemo(
    () => sessions.find((s) => s.id === activeId) ?? null,
    [sessions, activeId],
  );

  useEffect(() => save(ns, sessions), [ns, sessions]);
  useEffect(() => {
    scrollRef.current?.scrollTo(0, scrollRef.current.scrollHeight);
  }, [active?.messages.length, busy]);

  const newChat = () => {
    setActiveId(null);
    setInput("");
    setDrawer(false);
  };

  const deleteSession = (id: string) => {
    setSessions((prev) => {
      const next = prev.filter((s) => s.id !== id);
      if (id === activeId) setActiveId(next[0]?.id ?? null);
      return next;
    });
  };

  const appendMessage = (sessionId: string, msg: Message) => {
    setSessions((prev) =>
      prev.map((s) =>
        s.id === sessionId ? { ...s, messages: [...s.messages, msg] } : s,
      ),
    );
  };

  const send = async () => {
    const q = input.trim();
    if (!q || busy) return;
    setInput("");

    let sessionId = activeId;
    if (!sessionId) {
      sessionId = uid();
      const title = q.length > 24 ? q.slice(0, 24) + "…" : q;
      setSessions((prev) => [
        {
          id: sessionId!,
          title,
          messages: [{ role: "user", content: q }],
          createdAt: Date.now(),
        },
        ...prev,
      ]);
      setActiveId(sessionId);
    } else {
      appendMessage(sessionId, { role: "user", content: q });
    }

    setBusy(true);
    try {
      const a = await onAsk(q);
      appendMessage(sessionId, {
        role: "assistant",
        content: a.text,
        citations: a.citations,
      });
    } catch (e) {
      appendMessage(sessionId, {
        role: "assistant",
        content: `⚠️ 오류: ${e}`,
      });
    } finally {
      setBusy(false);
    }
  };

  const onKeyDown = (e: React.KeyboardEvent) => {
    // 모바일 키보드에서는 Enter 가 줄바꿈이어야 자연스러움 → 전송은 버튼으로만.
    if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      send();
    }
  };

  return (
    <div className="chatpane">
      <header className="chat-header">
        <button
          className="icon-btn"
          onClick={() => setDrawer(true)}
          aria-label="대화 목록"
        >
          ☰
        </button>
        <span className="chat-title">{active?.title ?? "새 채팅"}</span>
        <button className="icon-btn" onClick={newChat} aria-label="새 채팅">
          ✎
        </button>
      </header>
      {subtitle && <div className="chat-subtitle">{subtitle}</div>}

      <div className="messages" ref={scrollRef}>
        {!active && (
          <div className="placeholder">
            <p>무엇을 도와드릴까요?</p>
            <p className="note">
              업로드한 문서에 근거해서만 답변합니다. 근거가 없으면 “자료 없음”이라고 답합니다.
            </p>
          </div>
        )}
        {active?.messages.map((m, i) => (
          <div key={i} className={`msg ${m.role}`}>
            <div className="bubble">{m.content}</div>
            {m.citations && m.citations.length > 0 && (
              <ul className="citations">
                {m.citations.map((c, j) => (
                  <li key={j}>
                    <b>
                      {c.filename} p.{c.page}
                    </b>{" "}
                    {c.snippet}
                  </li>
                ))}
              </ul>
            )}
          </div>
        ))}
        {busy && (
          <div className="msg assistant">
            <div className="bubble typing">생각 중…</div>
          </div>
        )}
      </div>

      <div className="composer">
        <textarea
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={onKeyDown}
          rows={1}
          placeholder="예: D-2 비자에서 취업비자 전환 시 필요 서류는?"
        />
        <button
          className="send"
          onClick={send}
          disabled={busy || !input.trim()}
          aria-label="전송"
        >
          ↑
        </button>
      </div>

      {drawer && (
        <div className="drawer-backdrop" onClick={() => setDrawer(false)}>
          <aside className="drawer" onClick={(e) => e.stopPropagation()}>
            <button className="new-chat" onClick={newChat}>
              + 새 채팅
            </button>
            <ul className="session-list">
              {sessions.length === 0 && <li className="empty">채팅 기록 없음</li>}
              {sessions.map((s) => (
                <li
                  key={s.id}
                  className={s.id === activeId ? "active" : ""}
                  onClick={() => {
                    setActiveId(s.id);
                    setDrawer(false);
                  }}
                >
                  <span className="session-title">{s.title}</span>
                  <button
                    className="session-del"
                    aria-label="대화 삭제"
                    onClick={(e) => {
                      e.stopPropagation();
                      deleteSession(s.id);
                    }}
                  >
                    ×
                  </button>
                </li>
              ))}
            </ul>
          </aside>
        </div>
      )}
    </div>
  );
}
