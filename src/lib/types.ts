// Mirrors the Rust types serialized by isotope-core (camelCase JSON).

export type HibernationConfig = {
  enabled: boolean;
  timeoutMinutes: number;
  startHibernated: boolean;
};

export type App = {
  id: string;
  name: string;
  url: string;
  icon: string | null;
  profileId: string;
  userAgent: string | null;
  allowedDomains: string[];
  hibernation: HibernationConfig;
  notifications: boolean;
  badges: boolean;
};

export type NewApp = Omit<App, "id">;

export type Profile = { id: string; name: string };

export type Folder = { id: string; name: string; icon: string | null; appIds: string[] };

export type SidebarItem = { type: "app"; id: string } | { type: "folder"; id: string };

export type Container = { type: "sidebar" } | { type: "folder"; id: string };

export type Settings = { defaultHibernationMinutes: number; lastActiveAppId: string | null };

export type Config = {
  version: number;
  profiles: Profile[];
  apps: App[];
  folders: Folder[];
  sidebar: SidebarItem[];
  settings: Settings;
};

export type AppState =
  | { kind: "hibernated" }
  | { kind: "running" }
  | { kind: "active" }
  | { kind: "error"; message: string };

/** A count, or "dot" for "has unread" without a number. */
export type Badge = number | "dot";

export type Snapshot = {
  config: Config;
  states: Record<string, AppState>;
  badges: Record<string, Badge>;
  activeAppId: string | null;
  folderPanel: string | null;
  readOnly: boolean;
};

export type Toast = { level: "info" | "warning" | "error"; message: string };

export const DEFAULT_PROFILE_ID = "default";
