import { writable, derived } from 'svelte/store';
import translations from './translations';

type Translations = {
  [locale: string]: {
    [key: string]: string;
  };
};

const savedLocale: string = localStorage.getItem('locale') || 'en';

export const locale = writable<string>(savedLocale);

locale.subscribe((value) => {
  if (value) {
    localStorage.setItem('locale', value);
  }
});

export const locales: string[] = Object.keys(translations);

export function getAvailableLanguages(): string[] {
  return Object.keys(translations);
}

function translate(locale: string, key: string, vars: Record<string, string>): string {
  if (!key) throw new Error("No key provided to $t()");
  if (!locale) throw new Error(`No translation for key "${key}"`);

  let text = (translations as Translations)[locale][key];

  if (!text) throw new Error(`No translation found for ${locale}.${key}`);

  Object.keys(vars).forEach((k) => {
    const regex = new RegExp(`{{${k}}}`, "g");
    text = text.replace(regex, vars[k]);
  });

  return text;
}

export const t = derived(locale, ($locale) => (key: string, vars: Record<string, string> = {}) =>
  translate($locale, key, vars)
);