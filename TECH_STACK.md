# Sammy — Tech Stack & Architecture

_Brief, honest snapshot of what Sammy is actually built on today._
_Last updated: 2026-07-16_

---

## 1. The one-line idea

> **Separate the soul from the brain.**
> The model (brain) can change — local Kobold, Venice cloud, or a future model.
> The identity (soul) lives in an encrypted vault on your machine and travels with you.

---

## 2. Stack at a glance

| Layer | Technology | Why |
|---|---|---|
| **Shell / window** | Tauri 2 | Native desktop app, tiny footprint, Rust core |
| **UI** | React 19 + TypeScript + Tailwind v4 | Fast, modern, hot-reload during dev |
| **Bundler** | Vite 7 | Builds the UI; embedded into the exe for release |
| **Core logic** | Rust | Speed + memory safety = the security backbone |
| **Database** | SQLite via `rusqlite`, **encrypted with SQLCipher** | Whole DB is encrypted at rest |
| **File encryption** | AES-256-GCM (`aes-gcm`) | Encrypts imported documents (blobs) |
| **Key derivation** | Argon2id (`argon2`) | Turns your passphrase into a strong key |
| **Secret storage** | OS keychain (`keyring`, Windows-native) | API keys live in Windows Credential Manager, not in files |
| **HTTP / brains** | `reqwest` (rustls TLS) + `tokio` | Talks to Kobold / Venice, streams replies |
| **Doc extraction** | `zip` (.docx), `pdf-extract` (.pdf), native (.txt/.md/.csv/.html) | Pulls text out of your files |

**Not in the build (yet):** vector database, embeddings, Chroma. See §6.

---

## 3. Architecture — the trust gate

```
          ┌─────────────────────────────────────┐
          │            React UI (Tauri)          │
          │  Chat · Intake · Sources · Brain     │
          └───────────────────┬──────────────────┘
                              │  Tauri commands (IPC)
          ┌───────────────────▼──────────────────┐
          │              Rust core                │
          │                                       │
          │   ┌────────────┐   VAULT LOCKED?      │
          │   │  Unlock     │   Argon2id key      │
          │   │  gate       │   → SQLCipher DB     │
          │   └─────┬───────┘                     │
          │         │ (everything requires unlock)│
          │   ┌─────▼──────┬──────────┬─────────┐ │
          │   │  Facts     │ Sources  │  Chat   │ │
          │   │ "What I    │ (docs +  │ (turns) │ │
          │   │  know"     │  RAG)    │         │ │
          │   └────────────┴────┬─────┴────┬────┘ │
          └────────────────────┬───────────┬──────┘
                               │           │
                    build system prompt    │
                    (facts + doc context)  │
                               │           ▼
                               │     ┌───────────────┐
                               └────►│  Brain router  │
                                     │  prefer local, │
                                     │  fall back     │
                                     └──────┬─────────┘
                                            │
                        ┌───────────────────┴──────────────┐
                        ▼                                   ▼
                 Kobold (local :5001)              Venice (cloud API)
                 GPU on your box                    key in OS keychain
```

**Key rule:** Chat requires vault unlock. There is no "quick chat without memory" bypass — everything passes through the one gate. That's the trust model.

---

## 4. How memory works today

Two lanes, kept separate on purpose:

1. **Facts ("What I know")** — short, user-visible, editable, deletable, exportable.
   Injected into every chat turn's system prompt.

2. **Sources (documents)** — full files imported via **Chat → Attach** or the **Sources** tab.
   - Stored encrypted (AES-GCM), text extracted and chunked into the SQLCipher DB.
   - Included in every chat turn (whole library unless you focus on specific files).
   - **Small enough** → full text goes in. **Large** → sequential chunks (read in order),
     which beats keyword-guessing for "review this document" / life-history asks.

**Roller-coaster rule:** when your documents conflict with the model's pretrained guesses,
your documents win.

---

## 5. Brain swap (the "engine")

- Config file (`brains.toml`) lives **outside** the vault so brains can be listed before unlock.
- API keys live in the **OS keychain**, never in that config.
- Router: prefer local Kobold when it's serving; fall back to Venice; never hard-fail.
- **Beta tip:** to spare the GPU, select Venice and stop Kobold on `:5001` → cloud-only.

---

## 6. Where a vector DB / Chroma could fit (future)

Today's retrieval is **keyword + sequential** — no embeddings. That's fine for a handful
of documents. It won't scale to your **~1.2 GB personal corpus** (325 songs, books, letters).

When we build the corpus retrieval layer, the likely upgrade path is:

```
Raw archive (1.2 GB)  →  embeddings  →  vector store  →  semantic retrieval
                                         (Chroma, LanceDB,
                                          or SQLite-vec)
```

- **Chroma DB** — popular Python-native vector store; easy to prototype, would run as a
  sidecar service.
- **LanceDB / sqlite-vec** — embeddable, Rust-friendly, would keep everything in-process
  and inside the encrypted vault (better fit for Sammy's local-first, no-sidecar model).

**Decision not made yet.** Chroma is a candidate, not a commitment. For a fully local,
encrypted, single-exe app, an embeddable vector store (LanceDB / sqlite-vec) currently
looks like the cleaner match — but that's a call for when the corpus/extraction engine
is ready to marry into Sammy.

---

## 7. Build & run

- **Daily dev:** `npm run tauri dev` (from `D:\dev\Sammy`)
- **Standalone desktop exe:** `npm run tauri build -- --debug --no-bundle`
  → `src-tauri/target/debug/sammy.exe` (UI embedded, no dev server needed)
- See `BUILD_NOTES.md` for machine-specific gotchas (Perl for SQLCipher, Windows App Control).
