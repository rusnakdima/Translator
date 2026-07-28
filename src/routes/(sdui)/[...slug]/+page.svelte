<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { ThemeService, DynamicPage, SchemaShell, schemaLoader } from '@tauri-front/shared';

	let schema = null;
	let currentPageSchema = null;
	let loading = true;
	let error = null;

	async function loadSchema() {
		try {
			schema = await schemaLoader.loadFromUrl('/schemas/translatorschemas.json');
			updateCurrentPage($page.url.pathname);
		} catch (e) {
			error = e.message;
		} finally {
			loading = false;
		}
	}

	function updateCurrentPage(pathname: string) {
		if (!schema) return;
		currentPageSchema = schemaLoader.getPageByRoute(pathname);
	}

	onMount(() => {
		ThemeService.init();
		loadSchema();
	});

	$: updateCurrentPage($page.url.pathname);
</script>

{#if loading}
	<div class="min-h-screen flex items-center justify-center bg-base-200">
		<div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary"></div>
	</div>
{:else if error}
	<div class="min-h-screen flex items-center justify-center bg-base-200 p-6">
		<div class="bg-error/10 border border-error text-error px-6 py-4 rounded-xl max-w-md">
			<h2 class="font-bold text-lg mb-2">Schema Error</h2>
			<p>{error}</p>
			<button class="btn btn-primary mt-4" onclick={() => goto('/')}>Go Home</button>
		</div>
	</div>
{:else if schema && currentPageSchema}
	<SchemaShell layoutRegions={schema.layoutRegions || []} layoutMode={currentPageSchema.layoutMode || 'default'}>
		<DynamicPage schema={currentPageSchema} />
	</SchemaShell>
{:else if schema}
	<SchemaShell layoutRegions={schema.layoutRegions || []} layoutMode="default">
		<div class="flex items-center justify-center min-h-screen">
			<div class="text-center">
				<h1 class="text-4xl font-bold mb-4">404</h1>
				<p class="text-lg mb-4">Page not found</p>
				<button class="btn btn-primary" onclick={() => goto('/')}>Go Home</button>
			</div>
		</div>
	</SchemaShell>
{:else}
	<div class="min-h-screen flex items-center justify-center bg-base-200 p-6">
		<div class="text-center">
			<h1 class="text-4xl font-bold mb-4">404</h1>
			<p class="text-lg mb-4">No schema loaded</p>
			<button class="btn btn-primary" onclick={() => goto('/')}>Retry</button>
		</div>
	</div>
{/if}
