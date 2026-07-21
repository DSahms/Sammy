# Sammy → Corpus Project: Cross-Project Briefing

_A heads-up from the Sammy (platform) side to the Personal Knowledge Corpus (memory) side._
_These are two separate projects built in parallel. This doc exists so each knows the other._
_Last updated: 2026-07-16_

---

## Why you're reading this

Dave is building two things at once:

- **Sammy** — the secure platform / vehicle (this project). Chat, vault, brain-swap, docs.
- **Personal Knowledge Corpus** — the memory / soul (your project). Curated identity.

They ship separately but are designed to **marry later**. This is Sammy telling you what it
is and, crucially, **the seam where you'll plug in** — so you don't build something that has
to be re-fitted at the end.

> Guiding principle both sides share: **Separate the soul from the brain.**
> The model (brain) changes. The corpus (soul) stays. Sammy is the secure body that carries it.

---

## What Sammy is (so you can build compatibly)

- **Desktop app:** Tauri 2 + React + Rust core.
- **Everything is behind an encrypted vault:** Argon2id key derivation, SQLCipher database,
  AES-256-GCM for imported files. Chat does not run until the vault is unlocked. No bypass.
- **Brain layer is swappable:** local Kobold (`:5001`) preferred, Venice cloud fallback,
  router never hard-fails. API keys in the OS keychain.
- **Two memory lanes today:**
  1. **Facts ("What I know")** — short, user-visible, editable, deletable, exportable.
     Injected into every chat turn.
  2. **Sources (documents)** — full files, encrypted, chunked, retrieved into chat.
- **Trust rules (non-negotiable):** say "I don't know" rather than invent; confirm before
  storing memory; user documents beat pretrained guesses (roller-coaster rule); keep facts /
  suggestions / math in separate lanes.

Full detail: see `TECH_STACK.md` in the Sammy repo.

---

## The seam — where the corpus plugs into Sammy

Sammy builds a **system prompt** for every chat turn. Today it assembles:

```
[ trust rules ]  +  [ facts block ]  +  [ document context ]  →  brain
```

The corpus marries in **right here**, as a new curated layer that sits *above* raw documents:

```
        ┌──────────────────────────────────────────────┐
        │              Sammy system prompt              │
        │                                               │
        │  trust rules (Sammy owns these)               │
        │                                               │
        │  ┌─────────────────────────────────────────┐ │
        │  │  ALWAYS-ON IDENTITY  ◄── from corpus     │ │  ← small, curated
        │  │  WHO_DAVE_IS / VALUES / DECISION_FRAME / │ │
        │  │  COMMUNICATION_STYLE / WARNING_SYSTEM     │ │
        │  └─────────────────────────────────────────┘ │
        │                                               │
        │  ┌─────────────────────────────────────────┐ │
        │  │  ON-DEMAND DOMAIN  ◄── from corpus       │ │  ← retrieved by intent
        │  │  songs / poetry / business / timeline     │ │
        │  └─────────────────────────────────────────┘ │
        │                                               │
        │  facts block (Sammy owns)                     │
        │  document context (Sammy owns)                │
        └──────────────────────────────────────────────┘
```

So Sammy needs the corpus to expose **two kinds of output**:

### A. Always-on identity (small, curated, cheap)
- A handful of compressed markdown docs — e.g. `WHO_DAVE_IS.md`, `WHAT_DAVE_VALUES.md`,
  `DAVE_DECISION_FRAMEWORK.md`, `DAVE_COMMUNICATION_STYLE.md`, `DAVE_WARNING_SYSTEM.md`.
- These get injected into **every** turn. Keep them tight — this is prompt budget every message.
- This is the extraction-engine *output*, not raw material.

### B. On-demand domains (retrieved by intent)
- Larger bodies: 325 songs, poetry, books, business notes, life timeline.
- Loaded **only when relevant** ("help me write a song" → songwriting domain).
- This is where a vector store / embeddings will eventually live (Chroma, LanceDB, or
  sqlite-vec — undecided; Sammy's local-first design leans embeddable over sidecar).

---

## The integration contract (what to agree on now, cheaply)

To make the marriage clean instead of duct-taped, the two projects should settle a small
interface. Nothing needs to be built yet — just agreed:

1. **Format:** corpus outputs are **markdown** (identity docs) + a **manifest** (JSON) listing
   what's always-on vs on-demand, with tags/domains.
2. **Location:** corpus outputs land somewhere Sammy can read them — likely imported into the
   encrypted vault, OR a corpus folder Sammy ingests. (Decide: does the corpus live *inside*
   Sammy's vault, or as an external source Sammy pulls from?)
3. **Retrieval signal:** how does Sammy know which domain to load? Options: explicit user
   intent, a lightweight classifier, or corpus-provided tags. Start simple (tags / user pick).
4. **Curation boundary:** the corpus stores **distilled rules** ("Dave flags gambling as a
   vulnerability — check alignment before suggesting casinos"), not **raw logs** ("Dave went
   to Hard Rock on July 16"). Sammy's Sources tab holds raw imports until curated.
5. **Privacy:** corpus content is the most sensitive data in the system. It must ride inside
   Sammy's existing encryption (Argon2id + SQLCipher + AES-GCM), never plaintext on disk.

---

## What each side owns (don't step on each other)

| Sammy owns | Corpus owns |
|---|---|
| Vault, encryption, unlock gate | Extraction engine (raw → personality) |
| Chat, streaming, brain router | Curation decisions ("what represents Dave") |
| System-prompt assembly | Always-on identity docs |
| Document ingest + retrieval plumbing | Domain organization + tags/manifest |
| Trust rules enforcement | The "soul" content itself |

---

## Boundaries (same discipline Sammy was given)

- **Do not merge the two codebases.** They stay separate repos/projects until a deliberate,
  explicit integration step.
- **Do not pull in other products** (Persona-Lock, Arden, NURA, Consigliere, Pink Skies).
- The corpus is the **foundation**; Sammy (or another vehicle) is **one consumer** of it.
  Build the corpus so it could outlive any single app.

---

## One-paragraph summary to carry over

> Sammy is a Rust/Tauri secure desktop companion: encrypted vault, unlock-gated chat,
> swappable local/cloud brain, and document retrieval. It assembles a per-turn system prompt
> from trust rules + facts + document context. The corpus marries in by supplying two things:
> a **small always-on identity layer** (curated markdown, injected every turn) and
> **on-demand domain archives** (songs, poetry, business — retrieved by intent, eventually via
> embeddings). Agree the interface early — markdown + JSON manifest, stored inside Sammy's
> encryption, distilled rules not raw logs — so the two converge cleanly instead of being
> stitched together at the end.
