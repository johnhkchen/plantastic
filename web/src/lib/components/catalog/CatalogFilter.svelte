<script lang="ts">
	import type { Material } from '$lib/stores/project.svelte';
	import { CATEGORY_COLORS } from '$lib/styles/color-maps';

	const CATEGORIES = ['hardscape', 'softscape', 'edging', 'fill'] as const;
	const CATEGORY_LABELS: Record<string, string> = {
		hardscape: 'Hardscape',
		softscape: 'Softscape',
		edging: 'Edging',
		fill: 'Fill'
	};

	let {
		materials,
		onfilter
	}: {
		materials: Material[];
		onfilter: (filtered: Material[]) => void;
	} = $props();

	let searchQuery = $state('');
	let debouncedQuery = $state('');
	let activeCategory = $state<string | null>(null);

	// Debounce search input by 200ms
	let debounceTimer: ReturnType<typeof setTimeout> | undefined;
	$effect(() => {
		const q = searchQuery;
		clearTimeout(debounceTimer);
		debounceTimer = setTimeout(() => {
			debouncedQuery = q;
		}, 200);
		return () => clearTimeout(debounceTimer);
	});

	// Counts from the full (unfiltered) materials list
	let categoryCounts = $derived.by(() => {
		const counts: Record<string, number> = {};
		for (const cat of CATEGORIES) {
			counts[cat] = materials.filter((m) => m.category === cat).length;
		}
		return counts;
	});

	// Filter and emit
	$effect(() => {
		const query = debouncedQuery.toLowerCase();
		const cat = activeCategory;

		let result = materials;

		if (cat) {
			result = result.filter((m) => m.category === cat);
		}

		if (query) {
			result = result.filter((m) => m.name.toLowerCase().includes(query));
		}

		onfilter(result);
	});
</script>

<div class="mb-4 space-y-3">
	<!-- Search input -->
	<div class="relative">
		<svg
			class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-text-tertiary"
			fill="none"
			viewBox="0 0 24 24"
			stroke="currentColor"
			stroke-width="2"
		>
			<path
				stroke-linecap="round"
				stroke-linejoin="round"
				d="m21 21-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607Z"
			/>
		</svg>
		<input
			type="text"
			placeholder="Search materials…"
			bind:value={searchQuery}
			class="w-full rounded-md border border-border-strong py-2 pl-9 pr-3 text-sm shadow-sm placeholder:text-text-tertiary focus:border-primary focus:ring-1 focus:ring-primary"
		/>
	</div>

	<!-- Category tabs -->
	<div class="flex flex-wrap gap-2">
		<button
			type="button"
			class="rounded-full border px-3 py-1 text-xs font-medium transition-colors {activeCategory ===
			null
				? 'bg-surface-hover text-text border-border-strong'
				: 'border-transparent text-text-secondary hover:bg-surface-alt'}"
			onclick={() => (activeCategory = null)}
		>
			All ({materials.length})
		</button>
		{#each CATEGORIES as cat (cat)}
			{@const colors = CATEGORY_COLORS[cat]}
			{@const count = categoryCounts[cat]}
			{#if count > 0}
				<button
					type="button"
					class="rounded-full border px-3 py-1 text-xs font-medium transition-colors {activeCategory ===
					cat
						? colors.active
						: `border-transparent ${colors.inactive}`}"
					onclick={() => (activeCategory = activeCategory === cat ? null : cat)}
				>
					{CATEGORY_LABELS[cat]} ({count})
				</button>
			{/if}
		{/each}
	</div>
</div>
