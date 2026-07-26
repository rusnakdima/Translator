import { c as create_ssr_component, o as onDestroy, e as escape, b as add_attribute, d as each } from "../../chunks/ssr.js";
import { w as writable } from "../../chunks/index.js";
import { t } from "../../chunks/i18n.js";
import "@tauri-apps/api/core";
import "@tauri-apps/api/event";
const sourceLang = writable("en");
const targetLang = writable("ru");
const Page = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let charCount;
  let maxChars;
  let languages = [];
  let inputText = "";
  let translatedText = "";
  let sourceLangValue = "en";
  let targetLangValue = "ru";
  sourceLang.subscribe((v) => sourceLangValue = v);
  targetLang.subscribe((v) => targetLangValue = v);
  onDestroy(() => {
  });
  charCount = inputText.length;
  maxChars = 5e3;
  return `<div class="max-w-4xl mx-auto space-y-6"><div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6"><div class="grid grid-cols-[1fr_auto_1fr] gap-4 items-end"><div class="space-y-2"><label for="source-lang" class="block text-sm font-medium text-gray-700 dark:text-gray-300">${escape(t("translation.source"))}</label> <select id="source-lang"${add_attribute("value", sourceLangValue, 0)} class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500">${each(languages, (lang) => {
    return `<option${add_attribute("value", lang.code, 0)}>${escape(lang.name)}</option>`;
  })}</select></div> <button type="button" class="p-2 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"${add_attribute("title", t("translation.swap"), 0)} data-svelte-h="svelte-aa6brc">⇄</button> <div class="space-y-2"><label for="target-lang" class="block text-sm font-medium text-gray-700 dark:text-gray-300">${escape(t("translation.target"))}</label> <select id="target-lang"${add_attribute("value", targetLangValue, 0)} class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500">${each(languages, (lang) => {
    return `<option${add_attribute("value", lang.code, 0)}>${escape(lang.name)}</option>`;
  })}</select></div></div></div> <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6"><div class="grid grid-cols-1 md:grid-cols-2 gap-6"><div class="space-y-2"><div class="flex justify-between items-center"><label for="input-text" class="block text-sm font-medium text-gray-700 dark:text-gray-300">${escape(t("translation.input"))}</label> <span class="text-xs text-gray-500 dark:text-gray-400">${escape(t("translation.charCount", { count: charCount, max: maxChars }))}</span></div> <textarea id="input-text"${add_attribute("maxlength", maxChars, 0)}${add_attribute("rows", 8, 0)} class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 resize-none"${add_attribute("placeholder", t("translation.input"), 0)}>${escape("")}</textarea> <div class="flex gap-2"><button type="button" ${!inputText.trim() ? "disabled" : ""} class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed">${escape(t("translation.translate"))}</button> <button type="button" class="px-4 py-2 text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700">${escape(t("translation.clear"))}</button></div></div> <div class="space-y-2"><label for="output-text" class="block text-sm font-medium text-gray-700 dark:text-gray-300">${escape(t("translation.output"))}</label> <textarea id="output-text" readonly${add_attribute("rows", 8, 0)} class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-white resize-none"${add_attribute("placeholder", t("translation.output"), 0)}>${escape(translatedText, false)}</textarea> <button type="button" ${"disabled"} class="px-4 py-2 text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed">${escape(t("translation.copy"))}</button></div></div></div></div> ${``}`;
});
export {
  Page as default
};
