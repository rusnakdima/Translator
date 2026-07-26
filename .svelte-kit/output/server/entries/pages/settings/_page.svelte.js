import { c as create_ssr_component, e as escape } from "../../../chunks/ssr.js";
import { l as locale, t } from "../../../chunks/i18n.js";
import "@sveltejs/kit/internal";
import "../../../chunks/exports.js";
import "../../../chunks/utils.js";
import "@sveltejs/kit/internal/server";
import "../../../chunks/state.svelte.js";
const Page = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let currentLocale = "en";
  locale.subscribe((v) => currentLocale = v);
  return `<div class="max-w-2xl mx-auto"><div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6"><div class="flex items-center gap-4 mb-6"><button type="button" class="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors" data-svelte-h="svelte-lgre4d"><svg class="w-5 h-5 text-gray-600 dark:text-gray-300" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18"></path></svg></button> <h1 class="text-2xl font-bold text-gray-900 dark:text-white">${escape(t("settings.title"))}</h1></div> <div class="space-y-6"> <div class="border-b border-gray-200 dark:border-gray-700 pb-6"><h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">${escape(t("settings.language"))}</h2> <div class="flex gap-3"><button type="button" class="${"px-4 py-2 rounded-lg transition-colors " + escape(
    currentLocale === "en" ? "bg-blue-600 text-white" : "bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600",
    true
  )}">English</button> <button type="button" class="${"px-4 py-2 rounded-lg transition-colors " + escape(
    currentLocale === "ru" ? "bg-blue-600 text-white" : "bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600",
    true
  )}">Русский</button></div></div>  <div class="border-b border-gray-200 dark:border-gray-700 pb-6"><h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">${escape(t("settings.translation"))}</h2> <div class="bg-gray-50 dark:bg-gray-900 rounded-lg p-4"><div class="flex items-center gap-3" data-svelte-h="svelte-1chmckf"><div class="w-3 h-3 rounded-full bg-green-500"></div> <span class="text-gray-700 dark:text-gray-300">LibreTranslate</span></div> <p class="text-sm text-gray-500 dark:text-gray-400 mt-2">${escape(t("settings.translationDesc"))}</p></div></div>  <div><h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">${escape(t("settings.version"))}</h2> <div class="bg-gray-50 dark:bg-gray-900 rounded-lg p-4" data-svelte-h="svelte-tt07br"><p class="text-gray-700 dark:text-gray-300">Translator v1.0.0</p> <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">Powered by LibreTranslate</p></div></div></div></div></div>`;
});
export {
  Page as default
};
