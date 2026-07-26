<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { sourceLang, targetLang, swapLanguages } from "$lib/stores/globalState";
  import { locale, t } from "$lib/services/i18n";
  import { initShortcuts, cleanupShortcuts } from "$lib/services/shortcuts";
  import { getSupportedLanguages, translateText, listenForTranslationResult, type Language } from "$lib/services/translation";
  import { ToastHelper } from "$lib/utils/toast";
  import { ToastKind, RESPONSE_STATUS } from "$lib/utils/constants";

  let languages: Language[] = [];
  let inputText = "";
  let translatedText = "";
  let isTranslating = false;
  let showShortcuts = false;
  let unlisten: (() => void) | null = null;

  let sourceLangValue = "en";
  let targetLangValue = "ru";

  sourceLang.subscribe(v => sourceLangValue = v);
  targetLang.subscribe(v => targetLangValue = v);

  onMount(async () => {
    try {
      languages = await getSupportedLanguages();
    } catch (err) {
      ToastHelper.show(t("toast.translationError"), ToastKind.Error);
    }

    unlisten = (await listenForTranslationResult((payload) => {
      isTranslating = false;
      if (payload.response.status === RESPONSE_STATUS.success) {
        translatedText = payload.response.data.translatedText;
        ToastHelper.show(t("toast.translationComplete"), ToastKind.Success);
      } else {
        ToastHelper.show(t("toast.translationError"), ToastKind.Error);
      }
    })) as unknown as () => void;

    initShortcuts((action) => {
      switch (action) {
        case "translate":
          handleTranslate();
          break;
        case "swap":
          handleSwap();
          break;
        case "clear":
          handleClear();
          break;
        case "copy":
          handleCopy();
          break;
      }
    });

    window.addEventListener("onShortcutsOpen", () => {
      showShortcuts = true;
    });
  });

  onDestroy(() => {
    cleanupShortcuts();
    if (unlisten) unlisten();
  });

  async function handleTranslate() {
    if (!inputText.trim() || isTranslating) return;
    
    isTranslating = true;
    try {
      await translateText(inputText, sourceLangValue, targetLangValue);
    } catch (err) {
      isTranslating = false;
      ToastHelper.show(t("toast.translationError"), ToastKind.Error);
    }
  }

  function handleSwap() {
    swapLanguages();
  }

  function handleClear() {
    inputText = "";
    translatedText = "";
    ToastHelper.show(t("toast.cleared"), ToastKind.Info);
  }

  function handleCopy() {
    if (translatedText) {
      navigator.clipboard.writeText(translatedText);
      ToastHelper.show(t("toast.copied"), ToastKind.Success);
    }
  }

  function handleSourceChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    sourceLang.set(target.value);
  }

  function handleTargetChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    targetLang.set(target.value);
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      handleTranslate();
    }
  }

  function closeShortcuts() {
    showShortcuts = false;
  }

  $: charCount = inputText.length;
  $: maxChars = 5000;
</script>

<div class="max-w-4xl mx-auto space-y-6">
  <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6">
    <div class="grid grid-cols-[1fr_auto_1fr] gap-4 items-end">
      <div class="space-y-2">
        <label for="source-lang" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
          {t("translation.source")}
        </label>
        <select
          id="source-lang"
          value={sourceLangValue}
          on:change={handleSourceChange}
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
        >
          {#each languages as lang}
            <option value={lang.code}>{lang.name}</option>
          {/each}
        </select>
      </div>

      <button
        type="button"
        on:click={handleSwap}
        class="p-2 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
        title={t("translation.swap")}
      >
        ⇄
      </button>

      <div class="space-y-2">
        <label for="target-lang" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
          {t("translation.target")}
        </label>
        <select
          id="target-lang"
          value={targetLangValue}
          on:change={handleTargetChange}
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
        >
          {#each languages as lang}
            <option value={lang.code}>{lang.name}</option>
          {/each}
        </select>
      </div>
    </div>
  </div>

  <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6">
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
      <div class="space-y-2">
        <div class="flex justify-between items-center">
          <label for="input-text" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
            {t("translation.input")}
          </label>
          <span class="text-xs text-gray-500 dark:text-gray-400">
            {t("translation.charCount", { count: charCount, max: maxChars })}
          </span>
        </div>
        <textarea
          id="input-text"
          bind:value={inputText}
          on:keydown={handleKeydown}
          maxlength={maxChars}
          rows={8}
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 resize-none"
          placeholder={t("translation.input")}
        ></textarea>
        <div class="flex gap-2">
          <button
            type="button"
            on:click={handleTranslate}
            disabled={isTranslating || !inputText.trim()}
            class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isTranslating ? "..." : t("translation.translate")}
          </button>
          <button
            type="button"
            on:click={handleClear}
            class="px-4 py-2 text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"
          >
            {t("translation.clear")}
          </button>
        </div>
      </div>

      <div class="space-y-2">
        <label for="output-text" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
          {t("translation.output")}
        </label>
        <textarea
          id="output-text"
          value={translatedText}
          readonly
          rows={8}
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-white resize-none"
          placeholder={t("translation.output")}
        ></textarea>
        <button
          type="button"
          on:click={handleCopy}
          disabled={!translatedText}
          class="px-4 py-2 text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {t("translation.copy")}
        </button>
      </div>
    </div>
  </div>
</div>

{#if showShortcuts}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" on:click={closeShortcuts} on:keydown={(e) => e.key === "Escape" && closeShortcuts()} role="dialog" tabindex="-1">
    <div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl p-6 max-w-md w-full mx-4" on:click|stopPropagation role="document">
      <h2 class="text-xl font-bold text-gray-900 dark:text-white mb-4">{t("shortcuts.title")}</h2>
      <div class="space-y-2">
        <div class="flex justify-between text-sm">
          <span class="text-gray-600 dark:text-gray-400">Ctrl + Enter</span>
          <span class="text-gray-900 dark:text-white">{t("translation.translate")}</span>
        </div>
        <div class="flex justify-between text-sm">
          <span class="text-gray-600 dark:text-gray-400">Ctrl + Shift + S</span>
          <span class="text-gray-900 dark:text-white">{t("translation.swap")}</span>
        </div>
        <div class="flex justify-between text-sm">
          <span class="text-gray-600 dark:text-gray-400">Ctrl + K</span>
          <span class="text-gray-900 dark:text-white">{t("translation.clear")}</span>
        </div>
        <div class="flex justify-between text-sm">
          <span class="text-gray-600 dark:text-gray-400">Ctrl + Shift + C</span>
          <span class="text-gray-900 dark:text-white">{t("translation.copy")}</span>
        </div>
        <div class="flex justify-between text-sm">
          <span class="text-gray-600 dark:text-gray-400">F1</span>
          <span class="text-gray-900 dark:text-white">{t("shortcuts.title")}</span>
        </div>
      </div>
      <button
        type="button"
        on:click={closeShortcuts}
        class="mt-6 w-full px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
      >
        {t("shortcuts.close")}
      </button>
    </div>
  </div>
{/if}
