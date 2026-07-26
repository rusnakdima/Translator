<script lang="ts">
  import "../app.css";
  import { locale, t } from "$lib/services/i18n";

  let darkMode = false;

  function toggleDarkMode() {
    darkMode = !darkMode;
    document.documentElement.classList.toggle("dark", darkMode);
  }

  function toggleLocale() {
    locale.update(l => l === "en" ? "ru" : "en");
  }
</script>

<div class="min-h-screen bg-gray-50 dark:bg-gray-900">
  <header class="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-6 py-4">
    <div class="flex items-center justify-between">
      <h1 class="text-xl font-bold text-gray-900 dark:text-white">
        {t("app.title")}
      </h1>
      <div class="flex items-center gap-4">
        <button 
          type="button"
          on:click={toggleLocale}
          class="text-sm text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white"
        >
          {$locale === "en" ? "EN" : "RU"}
        </button>
        <button
          type="button"
          on:click={toggleDarkMode}
          class="text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white"
        >
          {darkMode ? "☀️" : "🌙"}
        </button>
      </div>
    </div>
  </header>

  <main class="p-6">
    <slot />
  </main>
</div>
