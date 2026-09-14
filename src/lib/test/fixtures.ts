import type { App, Snapshot } from "$lib/types";

export function app(id: string, overrides: Partial<App> = {}): App {
  return {
    id,
    name: id.toUpperCase(),
    url: `https://${id}.example.com`,
    icon: null,
    profileId: "default",
    userAgent: null,
    allowedDomains: [],
    hibernation: { enabled: true, timeoutMinutes: 10, startHibernated: false },
    notifications: true,
    badges: true,
    ...overrides,
  };
}

/** Apps a, b, c; folder f ("Work") holds c; sidebar [a, b, f]; a active; b has badge 3. */
export function snapshot(overrides: Partial<Snapshot> = {}): Snapshot {
  return {
    config: {
      version: 1,
      profiles: [{ id: "default", name: "Default" }],
      apps: [app("a"), app("b"), app("c")],
      folders: [{ id: "f", name: "Work", icon: null, appIds: ["c"] }],
      sidebar: [
        { type: "app", id: "a" },
        { type: "app", id: "b" },
        { type: "folder", id: "f" },
      ],
      settings: { defaultHibernationMinutes: 10, lastActiveAppId: "a" },
    },
    states: { a: { kind: "active" }, b: { kind: "running" }, c: { kind: "hibernated" } },
    badges: { b: 3 },
    activeAppId: "a",
    folderPanel: null,
    readOnly: false,
    ...overrides,
  };
}
