<script lang="ts">
	const TIERS = ['good', 'better', 'best'] as const;
	const TIER_LABELS: Record<string, string> = {
		good: 'Good',
		better: 'Better',
		best: 'Best'
	};

	let { activeTier = $bindable('good') }: { activeTier: string } = $props();
</script>

<div class="flex border-b border-border">
	{#each TIERS as tier (tier)}
		<button
			type="button"
			class="min-h-[44px] flex items-center justify-center px-4 py-2 text-sm font-medium transition-colors {activeTier === tier
				? 'border-b-2 border-primary text-primary'
				: 'text-text-secondary hover:text-text'}"
			onclick={() => (activeTier = tier)}
		>
			{TIER_LABELS[tier]}
		</button>
	{/each}
</div>
