# Sammy — "Got It" Kickoff (Locked Decisions)

**Purpose:** This is the locked architecture and scope for Sammy, already decided in a prior session. You are not being asked to propose these — you're being asked to **build against them**. If your build environment forces a genuine deviation (a tool, permission, or OS constraint that makes one of these literally impossible), flag it explicitly and propose the environment-appropriate equivalent — do not silently substitute something else.

**Companion file:** `SAMMY_EXPECTATIONS_ALIGNMENT.md` — the product requirements. Read that first for what Sammy is and does. This file is **how** it's built and in what order.

**Do not seek out other project files.** Do not look for, read, or reference any other files in this repo or environment — including anything named "Arden" (an earlier/related name for this same product line), "North Star," "Covenant," or founder-context docs — even if you find them. The two files provided here are the complete and only intended input for this build.

---

## 1. This is a fresh start — no drift

- Do not read, import, or reference any prior Persona-Lock, Arden, or related codebase in this environment.
- Any old screenshots, notes, or files from earlier attempts are reference-only if provided — do not inherit their structure, naming, or half-finished patterns.
- Build clean from the two files above.

---

## 2. Product boundary (do not conflate)

Sammy is its own product. Per the expectations doc's product family table:

- Do not merge in Consigliere, Witness, Pink Skies, NURA Connect, or Persona-Lock codebases or vaults.
- Persona-Lock encryption patterns may be a reference/lift source later — that is a deliberate, explicit decision if it happens, not an automatic merge.

---

## 3. Locked shell / stack

**Tauri 2 + React + Tailwind, Rust core.**

This is settled. Do not propose Electron, a pure web app, or a different frontend framework.

---

## 4. Locked scope — spine first

Build the spine before anything else:

1. **Vault** (encrypted storage)
2. **Brain swap** (swappable inference layer)
3. **Chat**
4. **Intake A/B/C** (per expectations doc: A = skip/get started now, B = quick get-to-know-you, C = deep intake, C includes B, always resumable)
5. **"What I know"** (user-visible facts, editable, deletable, exportable)

**RAG (Sources / document chat) is Cut 2** — not part of the initial build. This is a deliberate sequencing decision, not a deprioritization of importance. See the expectations doc — RAG/Sources is flagged as the highest-urgency gap, which is exactly why it gets its own focused cut rather than being bolted on mid-spine.

Do not let RAG, document upload, or "talk to your documents" work start until the spine above is complete and confirmed.

---

## 5. Architecture decisions carried forward from the locked build

These were confirmed and should carry forward as defaults. If your environment has a reason to do one differently, say so **before** proceeding — don't just change it.

- **Vault encryption:** encrypted at rest from the start (this was a stated gap/intent in the expectations doc — treat it as **in-scope for this build**, not deferred). Argon2id for key derivation, AES-256-GCM for source files, SQLCipher for the database itself.
- **Brain layer:** a trait/interface pattern so brains are swappable — implementations for at least one cloud brain (e.g. Venice) and one local brain (e.g. Ollama/Kobold), with a router that prefers local and falls back to cloud, never hard-fails.
- **Brain configs:** stored **outside** the encrypted vault (e.g. a config file) so available brains can be listed before unlock; actual API keys go in the **OS keychain**, not in that config file.
- **Chat requires vault unlock.** No "quick chat without memory" degraded mode. This is intentional — it keeps the trust model (everything passes through the one gate) intact. Do not add a bypass.
- **Sync:** design the schema/log now if convenient, but do not build the runtime — that's a later cut.
- **Theme:** dark by default. Light mode/theming is a later polish item, not a blocker now.

---

## 6. Trust rules (non-negotiable — from expectations doc)

- Say "I don't know" when something isn't in facts/memory/docs. Never invent.
- Confirm before storing new memory ("Should I remember...?").
- "What I know about you" must be user-visible, editable, deletable, exportable.
- No sycophantic fake corrections.
- Keep facts, suggestions, and math in separate lanes — don't blur confidence levels.

---

## 7. Process — how to work this build

1. **State your full plan** — file/module structure, what each piece does — **before writing any code.** This is the intercept point: if something is wrong, it gets caught here, not three files in.
2. Once the plan is confirmed, proceed in batches with milestone check-ins (e.g. "core compiles," "frontend builds"), not silent long stretches.
3. If you hit an environment limitation (missing system dependency, permission issue, etc.), say so plainly and propose the workaround — don't quietly restructure the architecture to route around it without flagging that you did.
4. If, mid-build, you find yourself wanting to revisit something marked locked above (the stack, the spine-first scope, RAG-as-cut-2, encryption-now, no-quick-chat), **stop and ask** rather than reasoning your way to a quiet change. A locked decision being inconvenient is not, by itself, a reason to reopen it.

---

## 8. What "done" looks like for this cut

- **Vault:** create/unlock, encrypted at rest, verified.
- **Brain swap:** at least one cloud + one local brain working, switchable.
- **Chat:** functional, requires unlock, streams responses.
- **Intake A/B/C:** all three paths present, resumable, feeding into memory/facts.
- **"What I know":** viewable, editable, exportable.
- Core and frontend both build clean.

Anything beyond this list (RAG, sync runtime, mobile, theming beyond dark default) is **explicitly out of scope for this cut.**
