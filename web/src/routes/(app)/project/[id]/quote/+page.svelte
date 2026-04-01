<script lang="ts">
	import { fetchQuote } from '$lib/api/quotes';
	import type { Quote } from '$lib/api/quotes';
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

	$effect(() => {
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
			})
			.catch((e) => {
				error = e instanceof Error ? e.message : 'Failed to load quotes';
			})
			.finally(() => {
				loading = false;
			});
	});
</script>

<div class="flex flex-col gap-4">
	<div class="flex items-center justify-between">
		<h2 class="text-lg font-semibold text-gray-900">Quote Comparison</h2>
		{#if error}
			<span class="text-xs text-red-500">{error}</span>
		{/if}
	</div>

	<QuoteComparison {quotes} {loading} />
</div>
