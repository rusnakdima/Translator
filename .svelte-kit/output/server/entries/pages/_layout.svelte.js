import { c as create_ssr_component, a as subscribe, e as escape, b as add_attribute, v as validate_component } from "../../chunks/ssr.js";
import "@sveltejs/kit/internal";
import "../../chunks/exports.js";
import "../../chunks/utils.js";
import "@sveltejs/kit/internal/server";
import "../../chunks/state.svelte.js";
import { w as writable, d as derived } from "../../chunks/index.js";
function goto(url, opts = {}) {
  {
    throw new Error("Cannot call goto(...) on the server");
  }
}
function createThemeStore() {
  const { subscribe: subscribe2, set, update } = writable({
    mode: "system",
    isDark: false
  });
  function getSystemDark() {
    if (typeof window === "undefined")
      return false;
    return window.matchMedia("(prefers-color-scheme: dark)").matches;
  }
  function applyDark(isDark) {
    if (typeof document === "undefined")
      return;
    if (isDark) {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
  }
  return {
    subscribe: subscribe2,
    /** Initialize theme from system preference or stored preference */
    init() {
      const stored = typeof localStorage !== "undefined" ? localStorage.getItem("theme-mode") : null;
      const mode = stored || "system";
      const isDark = mode === "dark" || mode === "system" && getSystemDark();
      applyDark(isDark);
      set({ mode, isDark });
      if (typeof window !== "undefined") {
        window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", (e) => {
          update((state) => {
            if (state.mode === "system") {
              const newIsDark = e.matches;
              applyDark(newIsDark);
              return { ...state, isDark: newIsDark };
            }
            return state;
          });
        });
      }
    },
    /** Toggle between light and dark (not system) */
    toggle() {
      update((state) => {
        const newMode = state.isDark ? "light" : "dark";
        const newIsDark = newMode === "dark";
        applyDark(newIsDark);
        if (typeof localStorage !== "undefined") {
          localStorage.setItem("theme-mode", newMode);
        }
        return { mode: newMode, isDark: newIsDark };
      });
    },
    /** Set specific mode */
    setMode(mode) {
      const isDark = mode === "dark" || mode === "system" && getSystemDark();
      applyDark(isDark);
      if (typeof localStorage !== "undefined") {
        localStorage.setItem("theme-mode", mode);
      }
      update((state) => ({ mode, isDark }));
    },
    /** Get current state */
    getState() {
      let state = { mode: "system", isDark: false };
      subscribe2((s) => {
        state = s;
      })();
      return state;
    }
  };
}
createThemeStore();
const SUPPORTED_LOCALES = [
  { code: "en", name: "English", nativeName: "English" },
  { code: "ru", name: "Russian", nativeName: "Русский", isRTL: false },
  { code: "es", name: "Spanish", nativeName: "Español", isRTL: false },
  { code: "fr", name: "French", nativeName: "Français", isRTL: false },
  { code: "de", name: "German", nativeName: "Deutsch", isRTL: false },
  { code: "zh", name: "Chinese", nativeName: "中文", isRTL: false },
  { code: "ja", name: "Japanese", nativeName: "日本語", isRTL: false },
  { code: "ko", name: "Korean", nativeName: "한국어", isRTL: false },
  { code: "it", name: "Italian", nativeName: "Italiano", isRTL: false },
  { code: "pt", name: "Portuguese", nativeName: "Português", isRTL: false },
  { code: "ar", name: "Arabic", nativeName: "العربية", isRTL: true }
];
const DEFAULT_TRANSLATIONS = {
  // General
  "app.name": "Application",
  "app.loading": "Loading...",
  "app.error": "An error occurred",
  "app.retry": "Retry",
  "app.close": "Close",
  "app.save": "Save",
  "app.cancel": "Cancel",
  "app.delete": "Delete",
  "app.edit": "Edit",
  "app.add": "Add",
  // Navigation
  "nav.home": "Home",
  "nav.settings": "Settings",
  "nav.about": "About",
  // Settings
  "settings.title": "Settings",
  "settings.language": "Language",
  "settings.theme": "Theme",
  "settings.darkMode": "Dark Mode",
  "settings.lightMode": "Light Mode",
  "settings.translation": "Translation Engine",
  "settings.translationDesc": "Using LibreTranslate API for text translation",
  "settings.version": "Version",
  // About
  "about.title": "About",
  "about.version": "Version",
  "about.description": "About this application",
  "about.aboutText": "A free and open source translation application powered by LibreTranslate. Supports multiple languages with offline capabilities.",
  "about.features": "Features",
  "about.feature1": "Multiple language support",
  "about.feature2": "Dark mode support",
  "about.feature3": "Offline translation ready",
  "about.feature4": "Free and open source",
  "about.poweredBy": "Powered By",
  // Errors
  "error.pageNotFound": "Page not found",
  "error.schemaError": "Schema error",
  "error.networkError": "Network error. Please check your connection.",
  "error.offline": "You are offline. Some features may not work.",
  "error.libraryRequired": "This feature requires the desktop library to be installed.",
  "error.installLibrary": "Install Library",
  "error.libraryDescription": "The translation library is not installed. Would you like to install it for offline use?",
  // Language names
  "lang.en": "English",
  "lang.ru": "Russian",
  "lang.es": "Spanish",
  "lang.fr": "French",
  "lang.de": "German",
  "lang.zh": "Chinese",
  "lang.ja": "Japanese",
  "lang.ko": "Korean",
  "lang.it": "Italian",
  "lang.pt": "Portuguese",
  "lang.ar": "Arabic",
  "lang.auto": "Detect language"
};
const RU_TRANSLATIONS = {
  "app.name": "Приложение",
  "app.loading": "Загрузка...",
  "app.error": "Произошла ошибка",
  "app.retry": "Повторить",
  "app.close": "Закрыть",
  "app.save": "Сохранить",
  "app.cancel": "Отмена",
  "app.delete": "Удалить",
  "app.edit": "Редактировать",
  "app.add": "Добавить",
  "nav.home": "Главная",
  "nav.settings": "Настройки",
  "nav.about": "О приложении",
  "settings.title": "Настройки",
  "settings.language": "Язык",
  "settings.theme": "Тема",
  "settings.darkMode": "Тёмный режим",
  "settings.lightMode": "Светлый режим",
  "settings.translation": "Движок перевода",
  "settings.translationDesc": "Используется API LibreTranslate для перевода текста",
  "settings.version": "Версия",
  "about.title": "О приложении",
  "about.version": "Версия",
  "about.description": "О приложении",
  "about.aboutText": "Бесплатное приложение для перевода с открытым исходным кодом на базе LibreTranslate. Поддерживает несколько языков с возможностью офлайн работы.",
  "about.features": "Возможности",
  "about.feature1": "Поддержка нескольких языков",
  "about.feature2": "Тёмная тема",
  "about.feature3": "Готово к офлайн переводу",
  "about.feature4": "Бесплатно и с открытым кодом",
  "about.poweredBy": "Технологии",
  "error.pageNotFound": "Страница не найдена",
  "error.schemaError": "Ошибка схемы",
  "error.networkError": "Ошибка сети. Проверьте подключение.",
  "error.offline": "Вы офлайн. Некоторые функции могут не работать.",
  "error.libraryRequired": "Для этой функции требуется установить настольную библиотеку.",
  "error.installLibrary": "Установить библиотеку",
  "error.libraryDescription": "Библиотека перевода не установлена. Хотите установить её для офлайн работы?",
  "lang.en": "Английский",
  "lang.ru": "Русский",
  "lang.es": "Испанский",
  "lang.fr": "Французский",
  "lang.de": "Немецкий",
  "lang.zh": "Китайский",
  "lang.ja": "Японский",
  "lang.ko": "Корейский",
  "lang.it": "Итальянский",
  "lang.pt": "Португальский",
  "lang.ar": "Арабский",
  "lang.auto": "Определить язык"
};
const ALL_TRANSLATIONS = {
  en: DEFAULT_TRANSLATIONS,
  ru: RU_TRANSLATIONS,
  es: {},
  fr: {},
  de: {},
  zh: {},
  ja: {},
  ko: {},
  it: {},
  pt: {},
  ar: {}
};
function createI18nStore() {
  const { subscribe: subscribe2, set, update } = writable({
    locale: "en",
    translations: DEFAULT_TRANSLATIONS,
    isRTL: false
  });
  return {
    subscribe: subscribe2,
    /** Set the current locale */
    setLocale(locale2) {
      const translations = ALL_TRANSLATIONS[locale2] || DEFAULT_TRANSLATIONS;
      const localeInfo = SUPPORTED_LOCALES.find((l) => l.code === locale2);
      const isRTL = localeInfo?.isRTL || false;
      if (typeof localStorage !== "undefined") {
        localStorage.setItem("i18n-locale", locale2);
      }
      set({ locale: locale2, translations, isRTL });
      if (typeof document !== "undefined") {
        document.documentElement.dir = isRTL ? "rtl" : "ltr";
        document.documentElement.lang = locale2;
      }
    },
    /** Get current locale */
    getLocale() {
      let locale2 = "en";
      subscribe2((s) => {
        locale2 = s.locale;
      })();
      return locale2;
    },
    /** Initialize from stored preference or browser */
    init() {
      const stored = typeof localStorage !== "undefined" ? localStorage.getItem("i18n-locale") : null;
      const browserLang = typeof navigator !== "undefined" ? navigator.language?.split("-")[0] : null;
      const initialLocale = stored || (browserLang && SUPPORTED_LOCALES.find((l) => l.code === browserLang) ? browserLang : "en");
      this.setLocale(initialLocale);
    },
    /** Get available locales */
    getAvailableLocales() {
      return SUPPORTED_LOCALES;
    },
    /** Check if locale has translations */
    hasTranslations(locale2) {
      return Object.keys(ALL_TRANSLATIONS[locale2] || {}).length > 0;
    }
  };
}
const i18n = createI18nStore();
const locale = derived(i18n, ($i18n) => $i18n.locale);
const Header_dropdown = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let currentLocale;
  let $locale, $$unsubscribe_locale;
  $$unsubscribe_locale = subscribe(locale, (value) => $locale = value);
  let { appName = "App" } = $$props;
  let { currentPath = "/" } = $$props;
  let { onSettings = void 0 } = $$props;
  let { onAbout = void 0 } = $$props;
  let dropdownRef;
  if ($$props.appName === void 0 && $$bindings.appName && appName !== void 0) $$bindings.appName(appName);
  if ($$props.currentPath === void 0 && $$bindings.currentPath && currentPath !== void 0) $$bindings.currentPath(currentPath);
  if ($$props.onSettings === void 0 && $$bindings.onSettings && onSettings !== void 0) $$bindings.onSettings(onSettings);
  if ($$props.onAbout === void 0 && $$bindings.onAbout && onAbout !== void 0) $$bindings.onAbout(onAbout);
  currentLocale = $locale;
  $$unsubscribe_locale();
  return `<header class="flex items-center justify-between px-6 py-4 bg-white dark:bg-gray-900 border-b border-gray-200 dark:border-gray-700"> <div class="flex items-center gap-4"><h1 class="text-xl font-bold text-gray-900 dark:text-white">${escape(appName)}</h1></div>  <div class="flex items-center gap-3"> <button type="button" class="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors" aria-label="Toggle theme">${`<svg class="w-5 h-5 text-gray-700" fill="currentColor" viewBox="0 0 20 20"><path d="M17.293 13.293A8 8 0 016.707 2.707a8.001 8.001 0 1010.586 10.586z"></path></svg>`}</button>  <div class="relative"${add_attribute("this", dropdownRef, 0)}><button type="button" class="flex items-center gap-2 px-3 py-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors text-gray-700 dark:text-gray-300"><svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 5h12M9 3v2m1.048 9.5A18.022 18.022 0 016.412 9m6.088 9h7M11 21l5-10 5 10M12.751 5C11.783 10.77 8.07 15.61 3 18.129"></path></svg> <span class="text-sm font-medium">${escape(currentLocale.toUpperCase())}</span> <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path></svg></button> ${``}</div></div></header>`;
});
function detectPlatform() {
  if (typeof window === "undefined")
    return "unknown";
  if (window.__TAURI__ || window.__TAURI_INTERNALS__) {
    return "tauri";
  }
  if (window.electron || window.process?.versions?.electron) {
    return "electron";
  }
  if (window.cordova || window.Capacitor) {
    return "tauri";
  }
  return "web";
}
function createPlatformStore() {
  const { subscribe: subscribe2, set, update } = writable({
    platform: "unknown",
    isOnline: true,
    isNativeApp: false
  });
  function init() {
    if (typeof window === "undefined")
      return;
    const platform = detectPlatform();
    const isNativeApp = platform === "tauri" || platform === "electron";
    const isOnline = navigator.onLine;
    set({ platform, isOnline, isNativeApp });
    window.addEventListener("online", () => {
      update((state) => ({ ...state, isOnline: true }));
    });
    window.addEventListener("offline", () => {
      update((state) => ({ ...state, isOnline: false }));
    });
  }
  return {
    subscribe: subscribe2,
    init,
    getPlatform: () => {
      let p = "unknown";
      subscribe2((s) => {
        p = s.platform;
      })();
      return p;
    },
    isOnline: () => {
      let o = true;
      subscribe2((s) => {
        o = s.isOnline;
      })();
      return o;
    }
  };
}
createPlatformStore();
const Layout = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let currentPath = "/";
  function handleSettings() {
    goto();
  }
  function handleAbout() {
    goto();
  }
  return `${validate_component(Header_dropdown, "HeaderDropdown").$$render(
    $$result,
    {
      appName: "Translator",
      currentPath,
      onSettings: handleSettings,
      onAbout: handleAbout
    },
    {},
    {}
  )} <main class="p-6">${slots.default ? slots.default({}) : ``}</main>`;
});
export {
  Layout as default
};
