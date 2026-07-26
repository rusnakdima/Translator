import { d as derived, w as writable } from "./index.js";
import { f as get_store_value } from "./ssr.js";
const translations = {
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
    "toast.cleared": "Cleared"
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
    "toast.cleared": "Очищено"
  }
};
const locale = writable("en");
const currentTranslations = derived(locale, ($locale) => translations[$locale]);
function t(key, params) {
  const dict = get_store_value(currentTranslations);
  let text = dict[key] || key;
  if (params) {
    Object.entries(params).forEach(([k, v]) => {
      text = text.replace(`{${k}}`, String(v));
    });
  }
  return text;
}
export {
  locale as l,
  t
};
