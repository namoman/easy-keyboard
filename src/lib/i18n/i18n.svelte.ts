import { browser } from "$app/environment";
import {
  MESSAGES,
  type LocaleId,
  type MessageKey,
} from "$lib/i18n/messages";

export type ThemeId = "default" | "contrast";

const LOCALE_KEY = "ek.locale";
const THEME_KEY = "ek.theme";

function readStored<T extends string>(key: string, fallback: T, ok: T[]): T {
  if (!browser) return fallback;
  try {
    const v = localStorage.getItem(key);
    return v && (ok as string[]).includes(v) ? (v as T) : fallback;
  } catch {
    return fallback;
  }
}

/** Minimal locale + optional theme hook (data-theme on <html>). */
class I18nStore {
  locale = $state<LocaleId>(readStored(LOCALE_KEY, "ko", ["ko", "en"]));
  theme = $state<ThemeId>(
    readStored(THEME_KEY, "default", ["default", "contrast"]),
  );

  constructor() {
    if (browser) this.applyTheme();
  }

  t(key: MessageKey): string {
    return MESSAGES[this.locale][key] ?? MESSAGES.en[key] ?? key;
  }

  setLocale(next: LocaleId): void {
    this.locale = next;
    if (browser) {
      try {
        localStorage.setItem(LOCALE_KEY, next);
      } catch {
        /* ignore quota */
      }
    }
  }

  setTheme(next: ThemeId): void {
    this.theme = next;
    if (browser) {
      try {
        localStorage.setItem(THEME_KEY, next);
      } catch {
        /* ignore */
      }
      this.applyTheme();
    }
  }

  private applyTheme(): void {
    document.documentElement.dataset.theme = this.theme;
  }
}

export const i18n = new I18nStore();
