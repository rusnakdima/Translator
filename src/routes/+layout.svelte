<script lang="ts">
	import "../app.css";
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { ThemeService, schemaLoader } from '@tauri-front/shared';

	let currentPath = '/';
	let schemaLoaded = false;
	let schemaError: string | null = null;

	onMount(async () => {
		ThemeService.init();

		const unsub = page.subscribe(p => {
			currentPath = p.url.pathname;
		});

		try {
			await schemaLoader.loadFromUrl('/schemas/translatorschemas.json');
			schemaLoaded = true;
		} catch (e: any) {
			schemaError = e.message;
			console.error('Failed to load schema:', e);
		}

		return unsub;
	});

	function handleSettings() {
		window.location.href = '/settings';
	}

	function handleAbout() {
		window.location.href = '/about';
	}

	function handleLanguageChange(lang: string) {
		// Language change handled by i18n service in pages
		console.log('Language change:', lang);
	}

	function handleLogoClick() {
		window.location.href = '/';
	}
</script>

{#if schemaError}
	<div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900">
		<div class="bg-red-100 dark:bg-red-900/30 border border-red-400 dark:border-red-600 text-red-700 dark:text-red-300 px-6 py-4 rounded-lg max-w-md">
			<p class="font-bold">Schema Load Error</p>
			<p>{schemaError}</p>
		</div>
	</div>
{:else if !schemaLoaded}
	<div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900">
		<div class="text-center">
			<div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
			<p class="text-gray-600 dark:text-gray-400">Loading schema...</p>
		</div>
	</div>
{:else}
	<slot />
{/if}
