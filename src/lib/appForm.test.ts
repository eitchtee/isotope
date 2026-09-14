import { expect, test } from "vitest";
import { formFromApp, normalizeUrl, validateAppForm } from "./appForm";
import { app } from "./test/fixtures";

test("normalizeUrl adds https and rejects non-web addresses", () => {
  expect(normalizeUrl("mail.google.com")).toBe("https://mail.google.com/");
  expect(normalizeUrl(" https://web.whatsapp.com ")).toBe("https://web.whatsapp.com/");
  expect(normalizeUrl("http://localhost:3000/app")).toBe("http://localhost:3000/app");
  expect(normalizeUrl("ftp://example.com")).toBeNull();
  expect(normalizeUrl("")).toBeNull();
  expect(normalizeUrl("not a url with spaces")).toBeNull();
});

test("new app form uses the defaults", () => {
  expect(formFromApp(undefined, { profileId: "default", timeoutMinutes: 15 })).toEqual({
    name: "",
    url: "",
    profileId: "default",
    userAgent: "",
    allowedDomains: "",
    hibernationEnabled: true,
    timeoutMinutes: 15,
    startHibernated: false,
    notifications: true,
    badges: true,
  });
});

test("edit form round-trips an app into the same NewApp", () => {
  const existing = app("a", {
    icon: "icons/a.png#1",
    userAgent: "UA",
    allowedDomains: ["login.microsoftonline.com", "okta.com"],
    hibernation: { enabled: false, timeoutMinutes: 3, startHibernated: true },
  });
  const form = formFromApp(existing, { profileId: "default", timeoutMinutes: 10 });

  const { input, errors } = validateAppForm(form, existing.icon);

  expect(errors).toEqual({});
  const { id: _id, ...rest } = existing;
  expect(input).toEqual({ ...rest, url: "https://a.example.com/" });
});

test("validation reports each invalid field and trims values", () => {
  const form = formFromApp(undefined, { profileId: "default", timeoutMinutes: 10 });
  form.name = "  ";
  form.url = "ftp://x";
  form.timeoutMinutes = 0;

  expect(validateAppForm(form, null)).toEqual({
    input: null,
    errors: {
      name: "Name is required",
      url: "Enter a web address (http or https)",
      timeoutMinutes: "Use a whole number of minutes, at least 1",
    },
  });

  form.name = " Gmail ";
  form.url = "mail.google.com";
  form.timeoutMinutes = 5;
  form.userAgent = "   ";
  form.allowedDomains = " accounts.google.com \n\n okta.com ";
  const { input } = validateAppForm(form, null);
  expect(input).toMatchObject({
    name: "Gmail",
    url: "https://mail.google.com/",
    userAgent: null,
    allowedDomains: ["accounts.google.com", "okta.com"],
    icon: null,
  });
});
