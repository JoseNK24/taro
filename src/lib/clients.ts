export const SUPPORTED_CLIENT_IDS = [
  "claude-code",
  "codex",
  "opencode",
  "cursor",
  "gemini-cli",
] as const;

export type SupportedClientId = (typeof SUPPORTED_CLIENT_IDS)[number];

export const CLIENT_LABELS: Record<SupportedClientId, string> = {
  "claude-code": "Claude",
  codex: "Codex",
  opencode: "OpenCode",
  cursor: "Cursor",
  "gemini-cli": "Gemini",
};

export const CLIENT_ORDER: SupportedClientId[] = [
  "claude-code",
  "codex",
  "opencode",
  "cursor",
  "gemini-cli",
];

const DRIVER_KIND_TO_CLIENT_ID: Record<string, SupportedClientId> = {
  claude_code: "claude-code",
  codex: "codex",
  opencode: "opencode",
  cursor: "cursor",
  gemini: "gemini-cli",
};

export function isSupportedClientId(id: string): id is SupportedClientId {
  return SUPPORTED_CLIENT_IDS.includes(id as SupportedClientId);
}

export function getClientLabel(clientId: string): string {
  if (isSupportedClientId(clientId)) {
    return CLIENT_LABELS[clientId];
  }
  return clientId;
}

export function clientIdFromDriverKind(driverKind: string): SupportedClientId | null {
  return DRIVER_KIND_TO_CLIENT_ID[driverKind] ?? null;
}

export function sortClientsByOrder<T extends { client_id: string }>(clients: T[]): T[] {
  return [...clients].sort((a, b) => {
    const ai = CLIENT_ORDER.indexOf(a.client_id as SupportedClientId);
    const bi = CLIENT_ORDER.indexOf(b.client_id as SupportedClientId);
    const aRank = ai === -1 ? Number.MAX_SAFE_INTEGER : ai;
    const bRank = bi === -1 ? Number.MAX_SAFE_INTEGER : bi;
    return aRank - bRank;
  });
}

export function filterSupportedClients<T extends { client_id: string }>(clients: T[]): T[] {
  return sortClientsByOrder(
    clients.filter((client) => isSupportedClientId(client.client_id)),
  );
}
