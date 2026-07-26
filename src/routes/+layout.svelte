<script lang="ts">
  import "../app.css";
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { HeaderDropdown, ThemeService } from '@tauri-front/shared';
  import { locale, setLocale } from '$lib/services/i18n';

  let currentPath = '/';

  onMount(() => {
    ThemeService.init();
    
    const unsub = page.subscribe(p => {
      currentPath = p.url.pathname;
    });
    return unsub;
  });

  function handleSettings() {
    goto('/settings');
  }

  function handleAbout() {
    goto('/about');
  }

  function handleLanguageChange(lang: string) {
    setLocale(lang as 'en' | 'ru');
  }

  function handleLogoClick() {
    goto('/');
  }
</script>

<HeaderDropdown
  appName="Translator"
  {currentPath}
  {locale}
  onSettings={handleSettings}
  onAbout={handleAbout}
  onLanguageChange={handleLanguageChange}
/>

<main class="p-6">
  <slot />
</main>
