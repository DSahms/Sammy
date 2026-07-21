import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  api,
  type BrainEntry,
  type ChatMessage,
  type Fact,
  type IntakeState,
  type SourceDoc,
  type VaultStatus,
} from "./lib/api";
import { open } from "@tauri-apps/plugin-dialog";

type Tab = "chat" | "intake" | "know" | "sources" | "brain";

async function pickAndAddSource(): Promise<SourceDoc | null> {
  const selected = await open({
    multiple: false,
    filters: [
      {
        name: "Documents",
        extensions: ["md", "txt", "csv", "pdf", "docx", "html", "htm"],
      },
    ],
  });
  if (!selected || Array.isArray(selected)) return null;
  return api.sourcesAdd(selected);
}

export default function App() {
  const [status, setStatus] = useState<VaultStatus | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [tab, setTab] = useState<Tab>("chat");
  const [passphrase, setPassphrase] = useState("");
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async () => {
    const s = await api.vaultStatus();
    setStatus(s);
  }, []);

  useEffect(() => {
    refresh().catch((e) => setError(String(e)));
  }, [refresh]);

  const onUnlock = async (create: boolean) => {
    setBusy(true);
    setError(null);
    try {
      const s = create
        ? await api.vaultCreate(passphrase)
        : await api.vaultUnlock(passphrase);
      setStatus(s);
      setPassphrase("");
      setTab("intake");
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const onLock = async () => {
    setBusy(true);
    try {
      setStatus(await api.vaultLock());
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  if (!status) {
    return (
      <div className="flex h-full items-center justify-center text-stone-400">
        Starting Sammy…
      </div>
    );
  }

  if (!status.unlocked) {
    return (
      <UnlockScreen
        exists={status.exists}
        path={status.path}
        passphrase={passphrase}
        setPassphrase={setPassphrase}
        busy={busy}
        error={error}
        onCreate={() => onUnlock(true)}
        onUnlock={() => onUnlock(false)}
      />
    );
  }

  return (
    <div className="flex h-full flex-col bg-[#0c1110]">
      <header className="flex items-center gap-3 border-b border-[#2a3830] px-4 py-3">
        <div className="text-lg font-semibold tracking-wide text-[#d4c9a0]">
          Sammy
        </div>
        <nav className="ml-4 flex gap-1">
          {(
            [
              ["chat", "Chat"],
              ["intake", "Know you"],
              ["sources", "Sources"],
              ["know", "What I know"],
              ["brain", "Brain"],
            ] as const
          ).map(([id, label]) => (
            <button
              key={id}
              onClick={() => setTab(id)}
              className={`rounded px-3 py-1.5 text-sm ${
                tab === id
                  ? "bg-[#1e3a2f] text-[#e8e4d4]"
                  : "text-stone-400 hover:bg-[#152019]"
              }`}
            >
              {label}
            </button>
          ))}
        </nav>
        <div className="ml-auto flex items-center gap-2">
          <button
            onClick={onLock}
            className="rounded border border-[#2a3830] px-3 py-1 text-xs text-stone-400 hover:bg-[#152019]"
          >
            Lock
          </button>
        </div>
      </header>
      {error && (
        <div className="border-b border-red-900/50 bg-red-950/40 px-4 py-2 text-sm text-red-200">
          {error}
          <button className="ml-3 underline" onClick={() => setError(null)}>
            dismiss
          </button>
        </div>
      )}
      <main className="min-h-0 flex-1">
        {tab === "chat" && <ChatPanel active onError={setError} />}
        {tab === "intake" && <IntakePanel onError={setError} />}
        {tab === "sources" && <SourcesPanel onError={setError} />}
        {tab === "know" && <KnowPanel onError={setError} />}
        {tab === "brain" && <BrainPanel onError={setError} />}
      </main>
    </div>
  );
}

function UnlockScreen(props: {
  exists: boolean;
  path: string;
  passphrase: string;
  setPassphrase: (s: string) => void;
  busy: boolean;
  error: string | null;
  onCreate: () => void;
  onUnlock: () => void;
}) {
  return (
    <div className="flex h-full items-center justify-center bg-[#0c1110] p-6">
      <div className="w-full max-w-md rounded-xl border border-[#2a3830] bg-[#121a16] p-8 shadow-xl">
        <h1 className="text-2xl font-semibold text-[#d4c9a0]">Sammy</h1>
        <p className="mt-2 text-sm text-stone-400">
          Persona-locked companion desk. Chat requires vault unlock — one gate,
          no bypass.
        </p>
        <p className="mt-3 text-xs text-stone-500 break-all">{props.path}</p>
        <label className="mt-6 block text-xs uppercase tracking-wider text-stone-500">
          Passphrase
        </label>
        <input
          type="password"
          className="mt-1 w-full rounded border border-[#2a3830] bg-[#0c1110] px-3 py-2 text-[#e8e4d4] outline-none focus:border-[#4a6b55]"
          value={props.passphrase}
          onChange={(e) => props.setPassphrase(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              props.exists ? props.onUnlock() : props.onCreate();
            }
          }}
          autoFocus
        />
        {props.error && (
          <p className="mt-3 text-sm text-red-300">{props.error}</p>
        )}
        <div className="mt-6 flex gap-2">
          {props.exists ? (
            <button
              disabled={props.busy || props.passphrase.length < 1}
              onClick={props.onUnlock}
              className="flex-1 rounded bg-[#2d5a45] px-4 py-2 font-medium text-[#e8e4d4] disabled:opacity-40"
            >
              Unlock
            </button>
          ) : (
            <button
              disabled={props.busy || props.passphrase.length < 8}
              onClick={props.onCreate}
              className="flex-1 rounded bg-[#2d5a45] px-4 py-2 font-medium text-[#e8e4d4] disabled:opacity-40"
            >
              Create vault
            </button>
          )}
        </div>
        {!props.exists && (
          <p className="mt-3 text-xs text-stone-500">
            At least 8 characters. Encrypted at rest (Argon2id + SQLCipher +
            AES-GCM ready for files).
          </p>
        )}
      </div>
    </div>
  );
}

function ChatPanel({ active, onError }: { active?: boolean; onError: (s: string | null) => void }) {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [draft, setDraft] = useState("");
  const [streaming, setStreaming] = useState("");
  const [busy, setBusy] = useState(false);
  const [docs, setDocs] = useState<SourceDoc[]>([]);
  const bottom = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  // Auto-grow the input line by line, capped so it never eats the screen
  useEffect(() => {
    const el = inputRef.current;
    if (!el) return;
    el.style.height = "auto";
    el.style.height = `${Math.min(el.scrollHeight, 200)}px`;
  }, [draft]);

  const refreshDocs = useCallback(() => {
    api.sourcesList().then(setDocs).catch(() => setDocs([]));
  }, []);

  useEffect(() => {
    api.chatHistory()
      .then(setMessages)
      .catch((e) => onError(String(e)));
    refreshDocs();
  }, [onError, refreshDocs]);

  // Refresh library when user opens Chat (e.g. after adding in Sources)
  useEffect(() => {
    if (active) refreshDocs();
  }, [active, refreshDocs]);

  useEffect(() => {
    let unsubs: Array<() => void> = [];
    (async () => {
      unsubs.push(
        await listen<string>("chat-chunk", (e) => {
          setStreaming((s) => s + e.payload);
        }),
      );
      unsubs.push(
        await listen<string>("chat-done", () => {
          setStreaming("");
        }),
      );
    })();
    return () => unsubs.forEach((u) => u());
  }, []);

  useEffect(() => {
    bottom.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, streaming]);

  const attach = async () => {
    setBusy(true);
    onError(null);
    try {
      const doc = await pickAndAddSource();
      if (doc) {
        refreshDocs();
        setMessages((m) => [
          ...m,
          {
            role: "assistant",
            content: `Added “${doc.filename}” to your library. It’s pinned for Chat — ask me to review it anytime.`,
          },
        ]);
      }
    } catch (e) {
      onError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const send = async () => {
    const text = draft.trim();
    if (!text || busy) return;
    setBusy(true);
    onError(null);
    setDraft("");
    setMessages((m) => [...m, { role: "user", content: text }]);
    setStreaming("");
    try {
      const reply = await api.chatSend(text);
      setMessages((m) => [...m, { role: "assistant", content: reply }]);
      setStreaming("");
    } catch (e) {
      onError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const pinned = docs.filter((d) => d.pinned);
  const inChat = docs.length > 0 ? docs : pinned;

  return (
    <div className="flex h-full flex-col">
      {inChat.length > 0 && (
        <div className="border-b border-[#2a3830] px-4 py-2 text-xs text-stone-400">
          Library in Chat:{" "}
          <span className="text-[#9cbaa4]">
            {inChat.map((d) => d.filename).join(", ")}
          </span>
          {pinned.length > 0 && pinned.length < docs.length && (
            <span className="text-stone-500">
              {" "}
              (focus: {pinned.length} of {docs.length} checked in Sources)
            </span>
          )}
        </div>
      )}
      <div className="min-h-0 flex-1 space-y-3 overflow-y-auto px-4 py-4">
        {messages.length === 0 && !streaming && (
          <p className="text-sm text-stone-500">
            Talk to Sammy. Attach a life-history or notes file — pinned Sources
            are included every turn. Then try “review this document for me.”
          </p>
        )}
        {messages.map((m, i) => (
          <div
            key={i}
            className={`max-w-3xl rounded-lg px-3 py-2 text-sm whitespace-pre-wrap ${
              m.role === "user"
                ? "ml-auto bg-[#1e3a2f] text-[#e8e4d4]"
                : "bg-[#152019] text-[#d4d0bc]"
            }`}
          >
            <div className="mb-1 text-[10px] uppercase tracking-wider text-stone-500">
              {m.role === "user" ? "You" : "Sammy"}
            </div>
            {m.content}
          </div>
        ))}
        {streaming && (
          <div className="max-w-3xl rounded-lg bg-[#152019] px-3 py-2 text-sm whitespace-pre-wrap text-[#d4d0bc]">
            <div className="mb-1 text-[10px] uppercase tracking-wider text-stone-500">
              Sammy
            </div>
            {streaming}
          </div>
        )}
        <div ref={bottom} />
      </div>
      <div className="flex items-end gap-2 border-t border-[#2a3830] px-3 pt-3 pb-6">
        <button
          onClick={attach}
          disabled={busy}
          className="rounded border border-[#2a3830] px-3 py-2 text-sm text-stone-300 hover:bg-[#152019] disabled:opacity-40"
        >
          Attach
        </button>
        <textarea
          ref={inputRef}
          rows={1}
          className="flex-1 resize-none rounded border border-[#2a3830] bg-[#0c1110] px-3 py-2 text-sm leading-relaxed outline-none focus:border-[#4a6b55]"
          placeholder="Message Sammy…  (Enter to send, Shift+Enter for a new line)"
          value={draft}
          disabled={busy}
          onChange={(e) => setDraft(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !e.shiftKey) {
              e.preventDefault();
              send();
            }
          }}
        />
        <button
          onClick={send}
          disabled={busy || !draft.trim()}
          className="rounded bg-[#2d5a45] px-4 py-2 text-sm font-medium disabled:opacity-40"
        >
          Send
        </button>
      </div>
    </div>
  );
}

function IntakePanel({ onError }: { onError: (s: string | null) => void }) {
  const [state, setState] = useState<IntakeState | null>(null);
  const [answer, setAnswer] = useState("");
  const [busy, setBusy] = useState(false);
  const [historyNote, setHistoryNote] = useState<string | null>(null);

  const load = useCallback(async () => {
    try {
      setState(await api.intakeState());
    } catch (e) {
      onError(String(e));
    }
  }, [onError]);

  useEffect(() => {
    load();
  }, [load]);

  const start = async (path: string) => {
    setBusy(true);
    try {
      setState(await api.intakeStart(path));
    } catch (e) {
      onError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const submit = async () => {
    if (!answer.trim()) return;
    setBusy(true);
    try {
      setState(await api.intakeAnswer(answer));
      setAnswer("");
    } catch (e) {
      onError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const addHistoryDoc = async () => {
    setBusy(true);
    onError(null);
    try {
      const doc = await pickAndAddSource();
      if (doc) {
        setHistoryNote(
          `Added “${doc.filename}” — Chat will use it every turn. You can keep going with A/B/C questions.`,
        );
      }
    } catch (e) {
      onError(String(e));
    } finally {
      setBusy(false);
    }
  };

  if (!state) {
    return <div className="p-6 text-stone-500">Loading…</div>;
  }

  return (
    <div className="mx-auto max-w-xl space-y-4 p-6">
      <h2 className="text-xl text-[#d4c9a0]">Help me know you</h2>
      <p className="text-sm text-stone-400">
        Never call this onboarding. A = get started now. B = quick. C = deep
        (includes B). Save & continue anytime.
      </p>
      <div className="rounded-lg border border-[#2a3830] bg-[#121a16] p-4">
        <p className="text-sm text-[#e8e4d4]">
          Have a life-history write-up (through 6th grade, or anything else)?
          Add it here — it becomes part of what Sammy knows in Chat.
        </p>
        <button
          disabled={busy}
          onClick={addHistoryDoc}
          className="mt-3 rounded bg-[#2d5a45] px-4 py-2 text-sm disabled:opacity-40"
        >
          Add life-history document…
        </button>
        {historyNote && (
          <p className="mt-2 text-sm text-[#9cbaa4]">{historyNote}</p>
        )}
      </div>
      <div className="flex flex-wrap gap-2">
        <button
          disabled={busy}
          onClick={() => start("A")}
          className="rounded border border-[#2a3830] px-3 py-2 text-sm hover:bg-[#152019]"
        >
          A · Get started
        </button>
        <button
          disabled={busy}
          onClick={() => start("B")}
          className="rounded border border-[#2a3830] px-3 py-2 text-sm hover:bg-[#152019]"
        >
          B · Quick
        </button>
        <button
          disabled={busy}
          onClick={() => start("C")}
          className="rounded border border-[#2a3830] px-3 py-2 text-sm hover:bg-[#152019]"
        >
          C · Help me know you
        </button>
      </div>
      <p className="text-xs text-stone-500">
        Path: {state.path} · step {state.step_index}/{state.total}
        {state.completed ? " · done" : ""}
      </p>
      {state.current && (
        <div className="rounded-lg border border-[#2a3830] bg-[#121a16] p-4">
          <p className="text-xs uppercase tracking-wider text-stone-500">
            {state.current.domain}
          </p>
          <p className="mt-2 text-[#e8e4d4]">{state.current.prompt}</p>
          <textarea
            className="mt-3 w-full rounded border border-[#2a3830] bg-[#0c1110] px-3 py-2 text-sm outline-none focus:border-[#4a6b55]"
            rows={3}
            value={answer}
            onChange={(e) => setAnswer(e.target.value)}
          />
          <button
            disabled={busy || !answer.trim()}
            onClick={submit}
            className="mt-3 rounded bg-[#2d5a45] px-4 py-2 text-sm disabled:opacity-40"
          >
            Save answer
          </button>
        </div>
      )}
      {state.completed && (
        <p className="text-sm text-[#9cbaa4]">
          Intake complete for now. You can resume or deepen later from here.
        </p>
      )}
    </div>
  );
}

function SourcesPanel({ onError }: { onError: (s: string | null) => void }) {
  const [docs, setDocs] = useState<SourceDoc[]>([]);
  const [busy, setBusy] = useState(false);

  const load = useCallback(async () => {
    try {
      setDocs(await api.sourcesList());
    } catch (e) {
      onError(String(e));
    }
  }, [onError]);

  useEffect(() => {
    load();
  }, [load]);

  const add = async () => {
    setBusy(true);
    onError(null);
    try {
      const doc = await pickAndAddSource();
      if (doc) await load();
    } catch (e) {
      onError(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="mx-auto max-w-2xl space-y-4 overflow-y-auto p-6">
      <div className="flex items-center justify-between">
        <h2 className="text-xl text-[#d4c9a0]">Sources</h2>
        <button
          disabled={busy}
          onClick={add}
          className="rounded bg-[#2d5a45] px-3 py-1.5 text-sm disabled:opacity-40"
        >
          Add file…
        </button>
      </div>
      <p className="text-sm text-stone-400">
        Same library as Chat Attach. Checked files are included every turn
        (default on). Uncheck only if you want to focus on specific files.
      </p>
      {docs.length === 0 && (
        <p className="text-sm text-stone-500">No documents yet.</p>
      )}
      {docs.map((d) => (
        <div
          key={d.id}
          className="flex items-start justify-between gap-3 rounded border border-[#2a3830] bg-[#121a16] px-3 py-2"
        >
          <label className="flex cursor-pointer items-start gap-2">
            <input
              type="checkbox"
              className="mt-1"
              checked={d.pinned}
              onChange={async (e) => {
                try {
                  await api.sourcesSetPinned(d.id, e.target.checked);
                  await load();
                } catch (err) {
                  onError(String(err));
                }
              }}
            />
            <div>
              <div className="font-medium text-[#e8e4d4]">{d.filename}</div>
              <div className="text-xs text-stone-500">
                {d.chunks} chunks · {(d.bytes / 1024).toFixed(1)} KB
              </div>
            </div>
          </label>
          <button
            className="text-xs text-red-300 hover:underline"
            onClick={async () => {
              try {
                await api.sourcesDelete(d.id);
                await load();
              } catch (err) {
                onError(String(err));
              }
            }}
          >
            Remove
          </button>
        </div>
      ))}
    </div>
  );
}

function KnowPanel({ onError }: { onError: (s: string | null) => void }) {
  const [facts, setFacts] = useState<Fact[]>([]);
  const [label, setLabel] = useState("");
  const [value, setValue] = useState("");
  const [domain, setDomain] = useState("preferences");
  const [busy, setBusy] = useState(false);

  const load = useCallback(async () => {
    try {
      setFacts(await api.factsList());
    } catch (e) {
      onError(String(e));
    }
  }, [onError]);

  useEffect(() => {
    load();
  }, [load]);

  const add = async () => {
    if (!label.trim() || !value.trim()) return;
    setBusy(true);
    try {
      await api.factsUpsert({ domain, label, value });
      setLabel("");
      setValue("");
      await load();
    } catch (e) {
      onError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const remove = async (id: string) => {
    setBusy(true);
    try {
      await api.factsDelete(id);
      await load();
    } catch (e) {
      onError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const exp = async () => {
    try {
      const path = await api.factsExport();
      onError(null);
      alert(`Exported to:\n${path}`);
    } catch (e) {
      onError(String(e));
    }
  };

  return (
    <div className="mx-auto flex h-full max-w-2xl flex-col gap-4 overflow-y-auto p-6">
      <div className="flex items-center justify-between">
        <h2 className="text-xl text-[#d4c9a0]">What I know about you</h2>
        <button
          onClick={exp}
          className="rounded border border-[#2a3830] px-3 py-1.5 text-xs hover:bg-[#152019]"
        >
          Export
        </button>
      </div>
      <p className="text-sm text-stone-400">
        Visible, editable, deletable, exportable. Confirm before storing elsewhere
        in chat — intake answers are explicit consent. Full documents live in
        Sources / Attach — facts here are summaries only.
      </p>
      <div className="space-y-2">
        {facts.length === 0 && (
          <p className="text-sm text-stone-500">Nothing stored yet.</p>
        )}
        {facts.map((f) => (
          <div
            key={f.id}
            className="rounded border border-[#2a3830] bg-[#121a16] px-3 py-2"
          >
            <div className="flex items-start justify-between gap-2">
              <div>
                <div className="text-xs text-stone-500">
                  {f.domain} · {f.source}
                </div>
                <div className="font-medium text-[#e8e4d4]">{f.label}</div>
                <div className="mt-1 text-sm text-stone-300 whitespace-pre-wrap">
                  {f.value}
                </div>
              </div>
              <button
                disabled={busy}
                onClick={() => remove(f.id)}
                className="text-xs text-red-300 hover:underline"
              >
                Delete
              </button>
            </div>
          </div>
        ))}
      </div>
      <div className="mt-auto rounded border border-[#2a3830] bg-[#121a16] p-3">
        <div className="text-xs uppercase tracking-wider text-stone-500">
          Add fact
        </div>
        <input
          className="mt-2 w-full rounded border border-[#2a3830] bg-[#0c1110] px-2 py-1.5 text-sm"
          placeholder="Domain"
          value={domain}
          onChange={(e) => setDomain(e.target.value)}
        />
        <input
          className="mt-2 w-full rounded border border-[#2a3830] bg-[#0c1110] px-2 py-1.5 text-sm"
          placeholder="Label"
          value={label}
          onChange={(e) => setLabel(e.target.value)}
        />
        <textarea
          className="mt-2 w-full rounded border border-[#2a3830] bg-[#0c1110] px-2 py-1.5 text-sm"
          placeholder="Value"
          rows={2}
          value={value}
          onChange={(e) => setValue(e.target.value)}
        />
        <button
          disabled={busy}
          onClick={add}
          className="mt-2 rounded bg-[#2d5a45] px-3 py-1.5 text-sm disabled:opacity-40"
        >
          Save
        </button>
      </div>
    </div>
  );
}

function BrainPanel({ onError }: { onError: (s: string | null) => void }) {
  const [brains, setBrains] = useState<BrainEntry[]>([]);
  const [active, setActive] = useState("");
  const [veniceKey, setVeniceKey] = useState("");
  const [hasVenice, setHasVenice] = useState(false);

  const load = useCallback(async () => {
    try {
      setBrains(await api.listBrains());
      setActive(await api.getActiveBrain());
      setHasVenice(await api.brainHasKey("venice"));
    } catch (e) {
      onError(String(e));
    }
  }, [onError]);

  useEffect(() => {
    load();
  }, [load]);

  return (
    <div className="mx-auto max-w-xl space-y-4 p-6">
      <h2 className="text-xl text-[#d4c9a0]">Brain</h2>
      <p className="text-sm text-stone-400">
        Keep the soul, change the engine. Prefer local Kobold at{" "}
        <code className="text-[#9cbaa4]">127.0.0.1:5001</code> when it’s serving;
        fall back to Venice. Config lives outside the vault; API keys in OS
        keychain.
      </p>
      <div className="space-y-2">
        {brains.map((b) => (
          <button
            key={b.id}
            onClick={async () => {
              try {
                setActive(await api.setActiveBrain(b.id));
              } catch (e) {
                onError(String(e));
              }
            }}
            className={`flex w-full items-center justify-between rounded border px-3 py-2 text-left text-sm ${
              active === b.id
                ? "border-[#4a6b55] bg-[#1e3a2f]"
                : "border-[#2a3830] bg-[#121a16] hover:bg-[#152019]"
            }`}
          >
            <span>
              <div className="font-medium">{b.label}</div>
              <div className="text-xs text-stone-500">{b.base_url}</div>
            </span>
            {active === b.id && (
              <span className="text-xs text-[#9cbaa4]">active</span>
            )}
          </button>
        ))}
      </div>
      <div className="rounded border border-[#2a3830] bg-[#121a16] p-3">
        <div className="text-xs uppercase tracking-wider text-stone-500">
          Venice API key {hasVenice ? "(saved)" : "(missing)"}
        </div>
        <input
          type="password"
          className="mt-2 w-full rounded border border-[#2a3830] bg-[#0c1110] px-2 py-1.5 text-sm"
          placeholder="Paste key — stored in OS keychain"
          value={veniceKey}
          onChange={(e) => setVeniceKey(e.target.value)}
        />
        <button
          className="mt-2 rounded bg-[#2d5a45] px-3 py-1.5 text-sm"
          onClick={async () => {
            try {
              await api.setBrainApiKey("venice", veniceKey);
              setVeniceKey("");
              setHasVenice(await api.brainHasKey("venice"));
            } catch (e) {
              onError(String(e));
            }
          }}
        >
          Save key
        </button>
      </div>
    </div>
  );
}
