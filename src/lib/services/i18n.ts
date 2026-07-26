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
    "settings.title": "Settings",
    "settings.language": "Language",
    "settings.theme": "Theme",
    "settings.darkMode": "Dark Mode",
    "settings.lightMode": "Light Mode",
    "settings.translation": "Translation Engine",
    "settings.translationDesc": "Using LibreTranslate API for text translation",
    "settings.version": "Version",
    "about.title": "About",
    "about.description": "About this application",
    "about.aboutText": "A free and open source translation application powered by LibreTranslate. Supports multiple languages with offline capabilities.",
    "about.features": "Features",
    "about.feature1": "Multiple language support",
    "about.feature2": "Dark mode support",
    "about.feature3": "Offline translation ready",
    "about.feature4": "Free and open source",
    "about.poweredBy": "Powered By",
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
    "settings.title": "Настройки",
    "settings.language": "Язык",
    "settings.theme": "Тема",
    "settings.darkMode": "Тёмный режим",
    "settings.lightMode": "Светлый режим",
    "settings.translation": "Движок перевода",
    "settings.translationDesc": "Используется API LibreTranslate для перевода текста",
    "settings.version": "Версия",
    "about.title": "О приложении",
    "about.description": "О приложении",
    "about.aboutText": "Бесплатное приложение для перевода с открытым исходным кодом на базе LibreTranslate. Поддерживает несколько языков с возможностью офлайн работы.",
    "about.features": "Возможности",
    "about.feature1": "Поддержка нескольких языков",
    "about.feature2": "Тёмная тема",
    "about.feature3": "Готово к офлайн переводу",
    "about.feature4": "Бесплатно и с открытым кодом",
    "about.poweredBy": "Технологии",
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
