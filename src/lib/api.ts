import { invoke } from "@tauri-apps/api/core";

export type VaultStatus = {
  exists: boolean;
  unlocked: boolean;
  path: string;
};

export type BrainEntry = {
  id: string;
  kind: string;
  label: string;
  base_url: string;
  model?: string | null;
  prefer?: boolean;
};

export type ChatMessage = {
  role: string;
  content: string;
};

export type IntakeQuestion = {
  id: string;
  domain: string;
  prompt: string;
};

export type IntakeState = {
  path: string;
  step_index: number;
  completed: boolean;
  current: IntakeQuestion | null;
  total: number;
};

export type Fact = {
  id: string;
  domain: string;
  label: string;
  value: string;
  source: string;
  created_at: string;
  updated_at: string;
};

export type SourceDoc = {
  id: string;
  filename: string;
  bytes: number;
  chunks: number;
  pinned: boolean;
  added_at: string;
};

export const api = {
  vaultStatus: () => invoke<VaultStatus>("vault_status"),
  vaultCreate: (passphrase: string) =>
    invoke<VaultStatus>("vault_create", { passphrase }),
  vaultUnlock: (passphrase: string) =>
    invoke<VaultStatus>("vault_unlock", { passphrase }),
  vaultLock: () => invoke<VaultStatus>("vault_lock"),
  listBrains: () => invoke<BrainEntry[]>("list_brains"),
  getActiveBrain: () => invoke<string>("get_active_brain"),
  setActiveBrain: (id: string) => invoke<string>("set_active_brain", { id }),
  setBrainApiKey: (id: string, apiKey: string) =>
    invoke<void>("set_brain_api_key", { id, apiKey }),
  brainHasKey: (id: string) => invoke<boolean>("brain_has_key", { id }),
  chatSend: (text: string) => invoke<string>("chat_send", { text }),
  chatHistory: () => invoke<ChatMessage[]>("chat_history"),
  intakeState: () => invoke<IntakeState>("intake_state"),
  intakeStart: (path: string) => invoke<IntakeState>("intake_start", { path }),
  intakeAnswer: (text: string) => invoke<IntakeState>("intake_answer", { text }),
  intakeSkipToChat: () => invoke<IntakeState>("intake_skip_to_chat"),
  factsList: () => invoke<Fact[]>("facts_list"),
  factsUpsert: (payload: {
    id?: string | null;
    domain: string;
    label: string;
    value: string;
  }) => invoke<Fact>("facts_upsert", payload),
  factsDelete: (id: string) => invoke<void>("facts_delete", { id }),
  factsExport: () => invoke<string>("facts_export"),
  sourcesList: () => invoke<SourceDoc[]>("sources_list"),
  sourcesAdd: (path: string) => invoke<SourceDoc>("sources_add", { path }),
  sourcesDelete: (id: string) => invoke<void>("sources_delete", { id }),
  sourcesSetPinned: (id: string, pinned: boolean) =>
    invoke<void>("sources_set_pinned", { id, pinned }),
};
