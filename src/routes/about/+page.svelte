<script lang="ts">
  import { t } from "$lib/services/i18n";
  import { goto } from "$app/navigation";
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
        <h1 class="text-2xl font-bold text-gray-900 dark:text-white">{t("about.title")}</h1>
      </div>
      
      <div class="space-y-6">
        <!-- App Info -->
        <div class="text-center py-8">
          <div class="w-20 h-20 mx-auto bg-blue-100 dark:bg-blue-900/30 rounded-2xl flex items-center justify-center mb-4">
            <svg class="w-10 h-10 text-blue-600 dark:text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 5h12M9 3v2m1.048 9.5A18.022 18.022 0 016.412 9m6.088 9h7M11 21l5-10 5 10M12.751 5C11.783 10.77 8.07 15.61 3 18.129" />
            </svg>
          </div>
          <h2 class="text-2xl font-bold text-gray-900 dark:text-white">Translator</h2>
          <p class="text-gray-500 dark:text-gray-400 mt-2">Version 1.0.0</p>
        </div>
        
        <!-- Description -->
        <div class="border-t border-gray-200 dark:border-gray-700 pt-6">
          <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-3">{t("about.description")}</h3>
          <p class="text-gray-600 dark:text-gray-300 leading-relaxed">
            {t("about.aboutText")}
          </p>
        </div>
        
        <!-- Features -->
        <div class="border-t border-gray-200 dark:border-gray-700 pt-6">
          <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-3">{t("about.features")}</h3>
          <ul class="space-y-2">
            <li class="flex items-center gap-2 text-gray-600 dark:text-gray-300">
              <svg class="w-5 h-5 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
              {t("about.feature1")}
            </li>
            <li class="flex items-center gap-2 text-gray-600 dark:text-gray-300">
              <svg class="w-5 h-5 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
              {t("about.feature2")}
            </li>
            <li class="flex items-center gap-2 text-gray-600 dark:text-gray-300">
              <svg class="w-5 h-5 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
              {t("about.feature3")}
            </li>
            <li class="flex items-center gap-2 text-gray-600 dark:text-gray-300">
              <svg class="w-5 h-5 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
              {t("about.feature4")}
            </li>
          </ul>
        </div>
        
        <!-- Powered By -->
        <div class="border-t border-gray-200 dark:border-gray-700 pt-6">
          <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-3">{t("about.poweredBy")}</h3>
          <div class="bg-gray-50 dark:bg-gray-900 rounded-lg p-4">
            <p class="font-medium text-gray-900 dark:text-white">LibreTranslate</p>
            <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">Free and Open Source Translation API</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>
