// 탭 A — 문서 관리. 모바일은 **유출가능 문서만** 업로드 가능.
// 파일 피커가 돌려주는 경로는 Android 에서 content:// URI 라 Rust 가 직접 못 읽는다.
// → 프론트에서 plugin-fs 로 바이트를 읽어 IPC 로 넘긴다.
import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { readFile } from "@tauri-apps/plugin-fs";
import {
  deleteDocument,
  indexDocument,
  listDocuments,
  onIndexProgress,
  uploadDocument,
} from "@/lib/ipc";
import { uploadRejectReason } from "@/lib/sensitivity";
import type { DocumentMeta } from "@/types";

const STAGE_LABEL: Record<string, string> = {
  pending: "대기",
  parsing: "텍스트 추출 중",
  chunking: "청킹 중",
  embedding: "임베딩 중",
  ready: "완료",
  error: "오류",
};

/** URI/경로 끝의 파일명 추출 (content:// 포함). */
function basename(p: string): string {
  const decoded = decodeURIComponent(p);
  const seg = decoded.split(/[\\/]/).pop() || decoded;
  return seg || "document.pdf";
}

export default function DocumentsTab() {
  const [docs, setDocs] = useState<DocumentMeta[]>([]);
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  const [progress, setProgress] = useState<Record<string, number>>({});

  const refresh = () =>
    listDocuments()
      .then(setDocs)
      .catch((e) => setErr(String(e)));

  useEffect(() => {
    refresh();
    const un = onIndexProgress((p) => {
      setProgress((prev) => ({ ...prev, [p.docId]: p.progress }));
      setDocs((prev) =>
        prev.map((d) =>
          d.id === p.docId ? { ...d, status: p.stage, progress: p.progress } : d,
        ),
      );
    });
    return () => {
      un.then((f) => f()).catch(() => {});
    };
  }, []);

  const onUpload = async () => {
    setErr(null);
    const picked = await open({
      multiple: false,
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
    if (!picked || typeof picked !== "string") return;

    setBusy(true);
    try {
      const bytes = await readFile(picked);
      // 모바일은 유출가능만 허용 — 태그 선택 UI 자체를 두지 않는다.
      const doc = await uploadDocument(basename(picked), bytes, "leakable");
      await refresh();
      try {
        await indexDocument(doc.id);
      } catch (e) {
        setErr(`인덱싱 실패(문서는 등록됨): ${e}`);
      }
      await refresh();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  const onReindex = async (id: string) => {
    setErr(null);
    setBusy(true);
    try {
      await indexDocument(id);
      await refresh();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  const onDelete = async (id: string) => {
    try {
      await deleteDocument(id);
      await refresh();
    } catch (e) {
      setErr(String(e));
    }
  };

  return (
    <section className="tab-section">
      <header className="tab-header">
        <h2>문서</h2>
        <button className="primary" disabled={busy} onClick={onUpload}>
          {busy ? "처리 중…" : "+ PDF 업로드"}
        </button>
      </header>

      <p className="warn">⚠️ {uploadRejectReason}</p>
      <p className="note">
        텍스트 PDF만 지원 (스캔 PDF·기타 포맷 미지원). 업로드 시 문서 내용이 임베딩을 위해
        OpenAI 로 전송됩니다.
      </p>

      {err && <p className="error">오류: {err}</p>}

      <ul className="doc-list">
        {docs.length === 0 && <li className="empty">업로드된 문서 없음</li>}
        {docs.map((d) => {
          const pct = Math.round((progress[d.id] ?? d.progress) * 100);
          const inFlight = d.status !== "ready" && d.status !== "error";
          return (
            <li key={d.id} className="doc-item">
              <div className="doc-main">
                <span className="doc-name">{d.filename}</span>
                <span className="doc-meta">
                  {d.status === "ready"
                    ? `${d.pages}p · ${d.chunkCount}청크`
                    : STAGE_LABEL[d.status] ?? d.status}
                  {inFlight && d.status !== "pending" ? ` ${pct}%` : ""}
                </span>
                {d.error && <span className="doc-err">{d.error}</span>}
                {inFlight && d.status !== "pending" && (
                  <div className="bar">
                    <div className="bar-fill" style={{ width: `${pct}%` }} />
                  </div>
                )}
              </div>
              <div className="doc-actions">
                {d.status !== "ready" && (
                  <button disabled={busy} onClick={() => onReindex(d.id)}>
                    재시도
                  </button>
                )}
                <button className="danger" onClick={() => onDelete(d.id)}>
                  삭제
                </button>
              </div>
            </li>
          );
        })}
      </ul>
    </section>
  );
}
