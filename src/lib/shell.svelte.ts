import { api, onStateChanged, onToast } from "./api";
import type { App, AppState, Badge, Folder, Snapshot, Toast } from "./types";

export type ToastEntry = Toast & { id: number };

const HIBERNATED: AppState = { kind: "hibernated" };

export class ShellStore {
  snapshot = $state<Snapshot | null>(null);
  toasts = $state<ToastEntry[]>([]);
  #nextToastId = 1;

  /** Pushes a command failure as an error toast; safe to pass directly to `.catch`. */
  reportError = (error: unknown) => {
    this.pushToast({ level: "error", message: String(error) });
  };

  /** Subscribes to Rust events, then loads the current state. Returns an unsubscribe function. */
  async start(): Promise<() => void> {
    const stopState = await onStateChanged((snapshot) => (this.snapshot = snapshot));
    const stopToasts = await onToast((toast) => this.pushToast(toast));
    this.snapshot = await api.getState();
    for (const toast of await api.takePendingToasts()) this.pushToast(toast);
    return () => {
      stopState();
      stopToasts();
    };
  }

  pushToast(toast: Toast) {
    this.toasts.push({ ...toast, id: this.#nextToastId++ });
  }

  dismissToast(id: number) {
    this.toasts = this.toasts.filter((t) => t.id !== id);
  }

  app(id: string): App | undefined {
    return this.snapshot?.config.apps.find((a) => a.id === id);
  }

  folder(id: string): Folder | undefined {
    return this.snapshot?.config.folders.find((f) => f.id === id);
  }

  state(id: string): AppState {
    return this.snapshot?.states[id] ?? HIBERNATED;
  }

  badge(id: string): Badge | undefined {
    return this.snapshot?.badges[id];
  }
}

export const shell = new ShellStore();
