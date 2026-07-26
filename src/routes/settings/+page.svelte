<script lang="ts">
  import { locale, t } from "$lib/services/i18n";
  import { goto } from "$app/navigation";
  import { ThemeService, themeStore } from "@tauri-front/shared";
  
  let currentLocale = "en";
  let isDark = false;
  
  locale.subscribe(v => currentLocale = v);
  
  const unsubTheme = themeStore.subscribe(state => {
    isDark = state.isDark;
  });
  
  function toggleDarkMode() {
    ThemeService.toggleDarkMode();
  }
</script>

<div class="min-h-screen bg-gray-50 dark:bg-gray-900">
  <div class="max-w-2xl mx-auto py-6">
    <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6">
      <div class="flex items-center gap-4 mb-6">
        <button
          type="button"
          on:click={() => goto('/')}
          class="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
        >
          <svg class="w-5 h-5 text-gray-600 dark:text-gray-300" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
          </svg>
        </button>
        <h1 class="text-2xl font-bold text-gray-900 dark:text-white">{t("settings.title")}</h1>
      </div>
      
      <div class="space-y-6">
        <!-- Theme Setting -->
        <div class="border-b border-gray-200 dark:border-gray-700 pb-6">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">{t("settings.theme")}</h2>
          <button
            type="button"
            on:click={toggleDarkMode}
            class="flex items-center gap-3 px-4 py-3 rounded-lg transition-colors bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600"
          >
            {#if isDark}
              <svg class="w-5 h-5 text-yellow-500" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M10 2a1 1 0 011 1v1a1 1 0 11-2 0V3a1 1 0 011-1zm4 8a4 4 0 11-8 0 4 4 0 018 0zm-.464 4.95l.707.707a1 1 0 001.414-1.414l-.707-.707a1 1 0 00-1.414 1.414zm2.12-10.607a1 1 0 010 1.414l-.706.707a1 1 0 11-1.414-1.414l.707-.707a1 1 0 011.414 0zM17 11a1 1 0 100-2h-1a1 1 0 100 2h1zm-7 4a1 1 0 011 1v1a1 1 0 11-2 0v-1a1 1 0 011-1zM5.05 6.464A1 1 0 106.465 5.05l-.708-.707a1 1 0 00-1.414 1.414l.707.707zm1.414 8.486l-.707.707a1 1 0 01-1.414-1.414l.707-.707a1 1 0 011.414 1.414zM4 11a1 1 0 100-2H3a1 1 0 000 2h1z" clip-rule="evenodd" />
              </svg>
              <span class="text-gray-700 dark:text-gray-300">{t("settings.darkMode")}</span>
            {:else}
              <svg class="w-5 h-5 text-gray-700" fill="currentColor" viewBox="0 0 20 20">
                <path d="M17.293 13.293A8 8 0 016.707 2.707a8.001 8.001 0 1010.586 10.586z" />
              </svg>
              <span class="text-gray-700 dark:text-gray-300">{t("settings.lightMode")}</span>
            {/if}
          </button>
        </div>
        
        <!-- Language Setting -->
        <div class="border-b border-gray-200 dark:border-gray-700 pb-6">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">{t("settings.language")}</h2>
          <div class="flex gap-3">
            <button
              type="button"
              on:click={() => locale.set("en")}
              class="px-4 py-2 rounded-lg transition-colors {currentLocale === 'en' 
                ? 'bg-blue-600 text-white' 
                : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'}"
            >
              English
            </button>
            <button
              type="button"
              on:click={() => locale.set("ru")}
              class="px-4 py-2 rounded-lg transition-colors {currentLocale === 'ru' 
                ? 'bg-blue-600 text-white' 
                : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'}"
            >
              Русский
            </button>
          </div>
        </div>
        
        <!-- Translation Engine -->
        <div class="border-b border-gray-200 dark:border-gray-700 pb-6">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">{t("settings.translation")}</h2>
          <div class="bg-gray-50 dark:bg-gray-900 rounded-lg p-4">
            <div class="flex items-center gap-3">
              <div class="w-3 h-3 rounded-full bg-green-500"></div>
              <span class="text-gray-700 dark:text-gray-300">LibreTranslate</span>
            </div>
            <p class="text-sm text-gray-500 dark:text-gray-400 mt-2">
              {t("settings.translationDesc")}
            </p>
          </div>
        </div>
        
        <!-- Version Section -->
        <div>
          <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">{t("settings.version")}</h2>
          <div class="bg-gray-50 dark:bg-gray-900 rounded-lg p-4">
            <p class="text-gray-700 dark:text-gray-300">Translator v1.0.0</p>
            <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">Powered by LibreTranslate</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>
