// 모바일 셸: 하단 탭바 3개(문서/채팅/설정).
// 데스크탑판의 상단 탭바 + 사이드바 패턴 대신 모바일 관례(하단 탭)를 따른다.
import { useState } from "react";
import DocumentsTab from "./tabs/DocumentsTab";
import ChatTab from "./tabs/ChatTab";
import SettingsTab from "./tabs/SettingsTab";

type Tab = "documents" | "chat" | "settings";

const TABS: { id: Tab; label: string; icon: string }[] = [
  { id: "documents", label: "문서", icon: "📄" },
  { id: "chat", label: "채팅", icon: "💬" },
  { id: "settings", label: "설정", icon: "⚙️" },
];

export default function App() {
  const [tab, setTab] = useState<Tab>("chat");

  return (
    <div className="app">
      <main className="tab-content">
        {/* 탭 전환 시 상태 초기화를 피하려고 언마운트 대신 표시만 토글 */}
        <div hidden={tab !== "documents"} className="pane">
          <DocumentsTab />
        </div>
        <div hidden={tab !== "chat"} className="pane">
          <ChatTab />
        </div>
        <div hidden={tab !== "settings"} className="pane">
          <SettingsTab />
        </div>
      </main>

      <nav className="tabbar" role="tablist">
        {TABS.map((t) => (
          <button
            key={t.id}
            role="tab"
            aria-selected={tab === t.id}
            className={tab === t.id ? "active" : ""}
            onClick={() => setTab(t.id)}
          >
            <span className="tab-icon" aria-hidden>
              {t.icon}
            </span>
            <span className="tab-label">{t.label}</span>
          </button>
        ))}
      </nav>
    </div>
  );
}
