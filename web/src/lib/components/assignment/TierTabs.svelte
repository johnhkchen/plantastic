<script lang="ts">
	const TIERS = ['good', 'better', 'best'] as const;
	const TIER_LABELS: Record<string, string> = {
		good: 'Good',
		better: 'Better',
		best: 'Best'
	};

	let { activeTier = $bindable('good') }: { activeTier: string } = $props();
</script>

<div class="flex border-b border-gray-200">
	{#each TIERS as tier (tier)}
		<button
			type="button"
			class="px-4 py-2 text-sm font-medium transition-colors {activeTier === tier
				? 'border-b-2 border-brand-primary text-brand-primary'
				: 'text-gray-500 hover:text-gray-700'}"
			onclick={() => (activeTier = tier)}
		>
			{TIER_LABELS[tier]}
		</button>
	{/each}
</div>
