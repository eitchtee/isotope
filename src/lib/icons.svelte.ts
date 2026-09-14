import { api } from "./api";

/**
 * Caches icon data URLs. The key includes the app's `icon` value, which Rust
 * changes (via a #version fragment) whenever the icon file is replaced.
 */
export class IconCache {
  #urls = $state<Record<string, string | null>>({});
  #pending = new Set<string>();

  url(appId: string, icon: string | null): string | null {
    if (!icon) return null;
    const key = `${appId}|${icon}`;
    if (!(key in this.#urls) && !this.#pending.has(key)) {
      this.#pending.add(key);
      api
        .appIcon(appId)
        .then((url) => (this.#urls[key] = url))
        .catch(() => (this.#urls[key] = null))
        .finally(() => this.#pending.delete(key));
    }
    return this.#urls[key] ?? null;
  }
}

export const icons = new IconCache();
