<script lang="ts">
	import type { Material } from '$lib/stores/project.svelte';
	import { formatPrice, formatUnit } from '$lib/utils/format';
	import { CATEGORY_HEADER_COLORS } from '$lib/styles/color-maps';

	let {
		materials,
		selectedZoneId,
		currentMaterialId,
		onAssign
	}: {
		materials: Material[];
		selectedZoneId: string | null;
		currentMaterialId: string | null;
		onAssign: (materialId: string) => void;
	} = $props();

	const CATEGORY_ORDER = ['hardscape', 'softscape', 'edging', 'fill'] as const;
	const CATEGORY_LABELS: Record<string, string> = {
		hardscape: 'Hardscape',
		softscape: 'Softscape',
		edging: 'Edging',
		fill: 'Fill'
	};

	let grouped = $derived.by(() => {
		const groups: Record<string, Material[]> = {};
		for (const cat of CATEGORY_ORDER) {
			const items = materials.filter((m) => m.category === cat);
			if (items.length > 0) groups[cat] = items;
		}
		return groups;
	});

	let disabled = $derived(!selectedZoneId);
</script>

<div class="flex h-full flex-col overflow-y-auto">
	{#if disabled}
		<div class="flex flex-1 items-center justify-center px-4">
			<p class="text-sm text-text-tertiary">Select a zone to assign materials</p>
		</div>
	{:else if materials.length === 0}
		<div class="flex flex-1 items-center justify-center px-4">
			<p class="text-sm text-text-tertiary">No materials in catalog.</p>
			<p class="mt-1 text-xs text-text-tertiary">Add materials in the Catalog page.</p>
		</div>
	{:else}
		{#each Object.entries(grouped) as [category, items] (category)}
			<div class="px-3 py-2">
				<h4 class="text-xs font-medium uppercase {CATEGORY_HEADER_COLORS[category] ?? 'text-text-secondary'}">
					{CATEGORY_LABELS[category] ?? category}
				</h4>
			</div>
			{#each items as mat (mat.id)}
				{@const isAssigned = currentMaterialId === mat.id}
				<button
					type="button"
					class="w-full border-t border-border-light px-3 py-2 text-left transition-colors {isAssigned
						? 'bg-primary-surface/15'
						: 'hover:bg-surface-alt'}"
					onclick={() => onAssign(mat.id)}
				>
					<div class="flex items-center justify-between">
						<span class="text-sm text-text">{mat.name}</span>
						<div class="flex items-center gap-2">
							<span class="text-xs text-text-secondary font-mono">
								{formatPrice(mat.price_per_unit)}/{formatUnit(mat.unit)}
							</span>
							{#if isAssigned}
								<span class="text-primary text-sm">&#10003;</span>
							{/if}
						</div>
					</div>
					{#if mat.supplier_sku}
						<div class="text-[11px] text-text-tertiary">SKU: {mat.supplier_sku}</div>
					{/if}
				</button>
			{/each}
		{/each}
	{/if}
</div>
