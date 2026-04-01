<script lang="ts">
	import type { Material } from '$lib/stores/project.svelte';
	import { formatPrice, formatUnit } from '$lib/utils/format';

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
	const CATEGORY_COLORS: Record<string, string> = {
		hardscape: 'text-stone-600',
		softscape: 'text-green-600',
		edging: 'text-amber-600',
		fill: 'text-orange-600'
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
			<p class="text-sm text-gray-400">Select a zone to assign materials</p>
		</div>
	{:else if materials.length === 0}
		<div class="flex flex-1 items-center justify-center px-4">
			<p class="text-sm text-gray-400">No materials in catalog.</p>
			<p class="mt-1 text-xs text-gray-400">Add materials in the Catalog page.</p>
		</div>
	{:else}
		{#each Object.entries(grouped) as [category, items] (category)}
			<div class="px-3 py-2">
				<h4 class="text-xs font-medium uppercase {CATEGORY_COLORS[category] ?? 'text-gray-500'}">
					{CATEGORY_LABELS[category] ?? category}
				</h4>
			</div>
			{#each items as mat (mat.id)}
				{@const isAssigned = currentMaterialId === mat.id}
				<button
					type="button"
					class="w-full border-t border-gray-50 px-3 py-2 text-left transition-colors {isAssigned
						? 'bg-brand-accent/15'
						: 'hover:bg-gray-50'}"
					onclick={() => onAssign(mat.id)}
				>
					<div class="flex items-center justify-between">
						<span class="text-sm text-gray-800">{mat.name}</span>
						<div class="flex items-center gap-2">
							<span class="text-xs text-gray-500 font-mono">
								{formatPrice(mat.price_per_unit)}/{formatUnit(mat.unit)}
							</span>
							{#if isAssigned}
								<span class="text-brand-primary text-sm">&#10003;</span>
							{/if}
						</div>
					</div>
					{#if mat.supplier_sku}
						<div class="text-[11px] text-gray-400">SKU: {mat.supplier_sku}</div>
					{/if}
				</button>
			{/each}
		{/each}
	{/if}
</div>
