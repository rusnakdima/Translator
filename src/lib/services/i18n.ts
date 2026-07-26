import { writable, derived, get } from "svelte/store";

export type Locale = "en" | "ru";

interface TranslationDict {
  [key: string]: string;
}

const translations: Record<Locale, TranslationDict> = {
  en: {
    "app.title": "Translator",
    "translation.source": "Source language",
    "translation.target": "Target language",
    "translation.input": "Enter text to translate...",
    "translation.output": "Translation",
    "translation.translate": "Translate",
    "translation.swap": "Swap languages",
    "translation.clear": "Clear",
    "translation.copy": "Copy",
    "translation.charCount": "{count} / {max} characters",
    "shortcuts.title": "Keyboard Shortcuts",
    "shortcuts.close": "Close",
    "toast.translationComplete": "Translation complete",
    "toast.translationError": "Translation failed",
    "toast.copied": "Copied to clipboard",
    "toast.cleared": "Cleared",
  },
  ru: {
    "app.title": "Переводчик",
    "translation.source": "Исходный язык",
    "translation.target": "Целевой язык",
    "translation.input": "Введите текст для перевода...",
    "translation.output": "Перевод",
    "translation.translate": "Перевести",
    "translation.swap": "Поменять языки",
    "translation.clear": "Очистить",
    "translation.copy": "Копировать",
    "translation.charCount": "{count} / {max} символов",
    "shortcuts.title": "Горячие клавиши",
    "shortcuts.close": "Закрыть",
    "toast.translationComplete": "Перевод завершен",
    "toast.translationError": "Ошибка перевода",
    "toast.copied": "Скопировано",
    "toast.cleared": "Очищено",
  },
};

export const locale = writable<Locale>("en");

export const currentTranslations = derived(locale, ($locale) => translations[$locale]);

export function setLocale(newLocale: Locale): void {
  locale.set(newLocale);
}

export function getAvailableLocales(): Locale[] {
  return ["en", "ru"];
}

export function t(key: string, params?: Record<string, string | number>): string {
  const dict = get(currentTranslations);
  let text = dict[key] || key;
  
  if (params) {
    Object.entries(params).forEach(([k, v]) => {
      text = text.replace(`{${k}}`, String(v));
    });
  }
  
  return text;
}
