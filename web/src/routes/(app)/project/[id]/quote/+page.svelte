<script lang="ts">
	import { fetchQuote } from '$lib/api/quotes';
	import type { Quote } from '$lib/api/quotes';
	import ErrorBanner from '$lib/components/ErrorBanner.svelte';
	import QuoteComparison from '$lib/components/quote/QuoteComparison.svelte';
	import type { LayoutData } from '../$types';

	let { data }: { data: LayoutData } = $props();
	let projectId = $derived(data.id);

	let quotes = $state<{ good: Quote | null; better: Quote | null; best: Quote | null }>({
		good: null,
		better: null,
		best: null
	});
	let loading = $state(true);
	let error = $state<string | null>(null);

	function loadQuotes() {
		const id = projectId;
		loading = true;
		error = null;
		quotes = { good: null, better: null, best: null };

		Promise.all([
			fetchQuote(id, 'good').catch(() => null),
			fetchQuote(id, 'better').catch(() => null),
			fetchQuote(id, 'best').catch(() => null)
		])
			.then(([good, better, best]) => {
				quotes = { good, better, best };
				if (!good && !better && !best) {
					error = 'Failed to load quotes';
				}
			})
			.catch((e) => {
				error = e instanceof Error ? e.message : 'Failed to load quotes';
			})
			.finally(() => {
				loading = false;
			});
	}

	$effect(() => {
		void projectId;
		loadQuotes();
	});
</script>

<div class="flex flex-col gap-4">
	<h2 class="text-lg font-semibold text-text">Quote Comparison</h2>

	{#if error}
		<ErrorBanner message={error} onretry={loadQuotes} />
	{/if}

	<QuoteComparison {quotes} {loading} />
</div>
