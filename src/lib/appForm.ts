import type { App, NewApp } from "./types";

export type AppForm = {
  name: string;
  url: string;
  profileId: string;
  userAgent: string;
  /** One domain per line. */
  allowedDomains: string;
  hibernationEnabled: boolean;
  timeoutMinutes: number;
  startHibernated: boolean;
  notifications: boolean;
  badges: boolean;
};

export type AppFormErrors = { name?: string; url?: string; timeoutMinutes?: string };

export function normalizeUrl(raw: string): string | null {
  const trimmed = raw.trim();
  if (!trimmed || /\s/.test(trimmed)) return null;
  const withScheme = /^[a-z][a-z0-9+.-]*:/i.test(trimmed) ? trimmed : `https://${trimmed}`;
  try {
    const url = new URL(withScheme);
    if ((url.protocol !== "https:" && url.protocol !== "http:") || !url.hostname) return null;
    return url.href;
  } catch {
    return null;
  }
}

export function formFromApp(app: App | undefined, defaults: { profileId: string; timeoutMinutes: number }): AppForm {
  return {
    name: app?.name ?? "",
    url: app?.url ?? "",
    profileId: app?.profileId ?? defaults.profileId,
    userAgent: app?.userAgent ?? "",
    allowedDomains: app?.allowedDomains.join("\n") ?? "",
    hibernationEnabled: app?.hibernation.enabled ?? true,
    timeoutMinutes: app?.hibernation.timeoutMinutes ?? defaults.timeoutMinutes,
    startHibernated: app?.hibernation.startHibernated ?? false,
    notifications: app?.notifications ?? true,
    badges: app?.badges ?? true,
  };
}

export function validateAppForm(form: AppForm, icon: string | null): { input: NewApp | null; errors: AppFormErrors } {
  const errors: AppFormErrors = {};
  const name = form.name.trim();
  const url = normalizeUrl(form.url);
  if (!name) errors.name = "Name is required";
  if (!url) errors.url = "Enter a web address (http or https)";
  if (!Number.isInteger(form.timeoutMinutes) || form.timeoutMinutes < 1) {
    errors.timeoutMinutes = "Use a whole number of minutes, at least 1";
  }
  if (Object.keys(errors).length > 0 || !url) return { input: null, errors };

  return {
    errors,
    input: {
      name,
      url,
      icon,
      profileId: form.profileId,
      userAgent: form.userAgent.trim() || null,
      allowedDomains: form.allowedDomains
        .split(/\s+/)
        .map((d) => d.trim())
        .filter(Boolean),
      hibernation: {
        enabled: form.hibernationEnabled,
        timeoutMinutes: form.timeoutMinutes,
        startHibernated: form.startHibernated,
      },
      notifications: form.notifications,
      badges: form.badges,
    },
  };
}
