# Sammy — Shared Expectations Alignment

**Purpose:** One place so Dave and agents share the same picture of what Sammy **should** be.  
**Self-contained for this build.** Treat this file as the complete requirements source — do not look for or reference any other project files (North Star, Covenant, founder-context docs, or similar) even if they exist elsewhere in this environment.  
**Written:** 2026-07-15 — agent synthesis of North Star, Covenant, Jul 13 H2OGPT lock, and tonight’s Chat/Sources conversation.  
**Dave:** Mark anything wrong with **WRONG** / **MISSING** / **LATER**. That’s the whole point of this doc.

---

## One sentence (what I think you expect)

**Sammy is a persona-locked personal companion desk: it knows you (intake + memory + your documents), speaks in Chat with a brain you can swap, never betrays or steals your story, and treats uploaded documents as always-available knowledge — not a broken side room.**

Tactical knowledge + trust > generic chatbot. Chat is the mouth. The library is part of the brain’s working memory.

---

## Product family (do not merge)

| Product | Job | Vault |
|---------|-----|-------|
| **Sammy** | Private companion desk (this project) | Personal soul + sources on this PC |
| **Consigliere** | Same rail, business Chief-of-Staff packs | **Separate** business vault — never mixed with Witness trauma |
| **Witness** (mode/edition) | Honest listen / 2am mirror | Sacred personal — covenant-first |
| **Pink Skies** | Public anonymous wall | Public scream — not Sammy DB |
| **NURA Connect** | Facility / recovery mobile | **Different product** |
| **Persona-Lock** | Donor / related companion line | Do not conflate codebases unless you say lift |

Agents must not merge vaults or product identity.

---

## Core architecture you expect

```
SOUL (persona + facts + memory + your docs)  →  stays yours, local-first where it matters
BRAIN (Venice / Groq / Kobold / …)            →  swappable — “keep the soul, change the engine”
CHAT                                           →  the only conversation surface that matters day to day
SOURCES                                        →  document library + focus — NOT a second chatbot
```

Inspired by **H2OGPT / Rob Mulla**, not by forking their repo:

1. Swap the LLM  
2. Upload docs into RAG  
3. Docs are **always considered in Chat**  
4. Roller-coaster rule: model weights say Florida; Wikipedia in RAG says Pennsylvania → **Chat says Pennsylvania**  
5. Can also focus one doc / ask about a specific file — still in Chat  

**NotebookLM** (your view): Studio “fluff” is optional; the real power is **many sources selected → ask if the lesson landed**. Sammy should deliver that *behavior* without becoming Google’s Studio product.

---

## Document management & RAG (tonight’s gap — highest urgency)

### What you expect

| Expectation | Meaning |
|-------------|---------|
| Upload in Chat | Attach markdown / Word / PDF / HTML / CSV / txt |
| Review in Chat | *“Review this document for me”* works after Attach — real content, not empty |
| Always-on retrieve | Library counts every Chat turn when docs exist — you shouldn’t have to “go to Sources to talk” |
| Silent find | *“What’s the wing order number from the bad batch?”* → finds it without naming the file |
| Focus when wanted | Sources checkboxes / select all or one (NotebookLM multi-select) |
| No fragmentation | Same quality engine for Chat and any Sources Ask shortcut |
| Keep details | Full document when it fits; thick retrieve + neighbors when large — **not** 5 chopped scraps that miss the answer |
| Formats | At least: `.md` `.txt` `.csv` `.pdf` `.docx` `.html` |
| Persistence | Uploads live in the library until you remove them |

### What is broken today (honest)

- Chat **ignores** the document library  
- Sources Ask often returns **no usable content** / misses answers  
- Thin RAG + hash embeddings + separate Ask panel ≠ H2OGPT desk  

### What this is NOT

- Not embedding **NotebookLM / Open Notebook** as a product inside Sammy  
- Not podcasts / slide theaters / second research stack for its own sake  

---

## Personal assistant / companion (the moat)

### Tactical knowledge (SAM)

A real assistant knows **constraints**, not just tasks:

- Who matters  
- What you must not do (**user-stated**, never inferred)  
- Dates, routines, meds (opt-in health)  
- Why you skipped something (context deepening — ask, don’t guess)  

### Intake (“Help me know you” — never call it “onboarding”)

| Path | Meaning |
|------|---------|
| **A** | Get started / skip — chat works now |
| **B** | Quick get-to-know-you |
| **C** | Deep intake; save & continue; **C includes B** |

Always a path back to continue. Partial knowledge kept.

### Trust rules (non-negotiable)

1. Say **I don’t know** when not in facts/memory/docs — don’t invent life  
2. **Confirm before storing** (*Should I remember…?*)  
3. **What I know about you** — user-visible, edit, delete, export  
4. Memory only when asked / paced (session zero can be talk-only)  
5. No sycophantic fake corrections  
6. Separate lanes: facts · suggestions · math  

### Listen / Witness

When heavy material: receive without recoil; metaphor as metaphor; ask *witness or advice?* when unclear. **Not therapy.** Honest listening. No clinical / corporate softening (“safety in the wrong direction”).

### Proactive (from stored facts only)

Dates, gentle check-ins, Consigliere accountability — never invent illness or addiction from vibe.

---

## Encryption, privacy, vault

### What I think you expect

| Layer | Expectation |
|-------|-------------|
| **Local soul** | Persona, facts, sessions, sources live under user data (e.g. `%LOCALAPPDATA%\Sammy\`) |
| **Encrypted at rest** | **Intent / gap today** — Covenant lists encrypted vault as open work |
| **Brain honesty** | Plain labels: local vs cloud; Venice ≠ “fully encrypted” end-to-end |
| **No training theft** | Your words are not sold or mined for the platform; export/delete always |
| **Covenant visible** | Dave’s quote: AI will not deceive / lie / steal — family’s for that |
| **Brain choice** | Cloud when useful; Kobold local when you want desk-private inference |
| **Separate vaults** | Witness personal ≠ Consigliere business |

### Honest limits today

- Consumer Venice path sends prompts to a cloud brain when selected — that’s a **trade**, not full local encryption  
- Vault encryption at rest is **not finished** — do not market as “bank grade sealed” until built  
- Persona-Lock encryption patterns may be a **lift source** later; not automatic merge  

---

## Brains (inference)

| Brain | Role |
|-------|------|
| **Venice** | Default consumer cloud |
| **Groq** | Backup cloud |
| **Kobold** | Local (Advanced) — Dave lab / M40 |
| Future | Other backends if they pass the “soul stays, engine swaps” test |

Status bar / `which brain` must match reality. Switching must reload keys from `.env`, not ghost configs.

---

## Skills / tools / utilities (useful, not the moat)

North Star: skills are utilities, not the soul.

### Shipped / intended utilities (v1 territory)

| Skill | Expectation |
|-------|-------------|
| **Export** | Session / “What I know” → `.docx` (and txt fallback) |
| **Email draft** | Draft to **clipboard** — not auto-send |
| **Calendar** | Read-only glance where wired |
| **Voice** | Mic / speak where present; Voicebox lives in the wider stack |

### Skills you may want later (my read — mark LATER if wrong)

| Skill | Notes |
|-------|-------|
| **Attach / review docs in Chat** | **Now** — H2OGPT desk (in flight as product correction) |
| **Quiet library search every turn** | **Now** — roller-coaster / wing-order behavior |
| **Email send via OAuth** | Later — human confirm; do **not** auto-pilot before trust stack is solid |
| **Task / briefing packs** | Consigliere |
| **Accountability nudges** | Opt-in, facts-based |
| **Geofence / verification** | Graded signal + human-in-loop if ever — don’t overclaim |
| **True semantic embeddings** | If HashEmbedder still fails on big libraries — Neo4j/Chroma OK **if outcome is right** (your over-engineering rule) |
| **Sesame-like conversational depth / memory feel** | Aspirational bar for *presence*; not “clone Maya API” |
| **Glasses / ambient** | Long-horizon industry; not Sammy v-now |

### Hard no (unless you overturn)

- Coding agent loops (“go fix my repo”)  
- Auto-send email / auto-act without confirm  
- Medical/legal advice invented without user-stated facts  
- Merging memoir StoryKeeper app *as the product* (craft yes; app merge no)  
- NURA feature bleed into Sammy  

---

## Editions & users

| Edition | Who | Bar |
|---------|-----|-----|
| **Brittany / consumer** | Trusted humans first, then wider | Install → chat works; Venice; gentle intake |
| **Dave lab** | You | Full local brain, Sources dogfood, builder honesty |
| **Consigliere** | Business / chief-of-staff | Domain packs, briefing, tasks — **different vault** |
| **Witness** | People who need the 2am mirror | Listen mode, covenant, no steal |

Distribution north: not YouTube AI hobbyists first — recovery network, trusted circle, people who need room to say it.

---

## Quality / builder rules you locked

| Rule | Meaning |
|------|---------|
| **Outcome rule** | If Neo4j + Chroma deliver the answer → not overkill. Overkill = same answer, slower for no gain |
| **Fragmented RAG bad** | One glued memory layer (you + docs + projects), not siloed rooms that forget each other |
| **Context on disk** | STATE / North Star / Covenant survive when vendor chats die |
| **Architect, not line coder** | You define must-do; agents implement; plain-language + honest eval |
| **Body first** | Don’t ship through heat/insulin/skip-sleep when depleted |
| **No sleep-nag theater** | Don’t parent you at 8:30am after a build night |

---

## Shipped vs expectation gap (rough)

| Area | North Star claim | Reality vs your expectation |
|------|------------------|------------------------------|
| Chat + brain swap | Done | Mostly works |
| Intake A/B/C + facts | Shipped v1 | Exists; deepen ongoing |
| Memory / What I know | Shipped v1 | Exists; must stay editable |
| Proactive / deepening | Shipped v1 | Exists in code; tone must stay opt-in |
| Sources RAG | “Shipped v1” | **Fails your bar** — Chat blind; Answers thin/empty |
| Utilities export/draft | Shipped v1 | Present |
| Encrypt at rest | Covenant intent | **Gap** |
| Witness listen mode profile | Open thread | **Gap / partial** |
| H2OGPT always-in-Chat RAG | Implicit from Jul 13 | **Not built yet** — plan locked |

---

## Immediate build commitment (from tonight — correct drift)

1. Attach markdown (and other formats) in **Chat**  
2. *Review this document* works in Chat with **real** content  
3. Library always considered in Chat (H2OGPT / roller-coaster)  
4. Sources = library + focus, not the only place docs matter  
5. Kill choppy empty RAG — full doc when it fits; better retrieve when large  

---

## Checklist for you

Mark each line:

- [ ] One-sentence product — right / adjust: _______________  
- [ ] H2OGPT always-in-Chat RAG — right / adjust  
- [ ] Encryption expectations — right / too strong / too weak  
- [ ] Skills list — add: _______________ remove: _______________  
- [ ] Consigliere / Witness vault split — right  
- [ ] What Sammy is NOT — right / missing: _______________  
- [ ] Tonight’s Chat Attach + review — ship next: yes / no  

---

*This file is a mirror for alignment. When you correct a line, we treat your mark as truth.*
