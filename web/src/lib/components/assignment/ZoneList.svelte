<script lang="ts">
	import type { ApiZone } from '$lib/api/types';
	import type { AssignmentResponse } from '$lib/api/tiers';
	import type { Material } from '$lib/stores/project.svelte';
	import { ZONE_BADGE_COLORS } from '$lib/styles/color-maps';

	let {
		zones,
		selectedZoneId = $bindable(null),
		assignments,
		materials
	}: {
		zones: ApiZone[];
		selectedZoneId: string | null;
		assignments: AssignmentResponse[];
		materials: Material[];
	} = $props();

	function getAssignedMaterial(zoneId: string): string | null {
		const a = assignments.find((a) => a.zone_id === zoneId);
		if (!a) return null;
		const mat = materials.find((m) => m.id === a.material_id);
		return mat?.name ?? null;
	}
</script>

<div class="flex h-full flex-col overflow-y-auto">
	<div class="px-3 py-2">
		<h3 class="text-xs font-medium uppercase text-text-secondary">Zones</h3>
	</div>
	{#if zones.length === 0}
		<div class="px-3 py-6 text-center">
			<p class="text-sm text-text-tertiary">No zones yet.</p>
			<p class="mt-1 text-xs text-text-tertiary">Draw zones in the Editor tab first.</p>
		</div>
	{:else}
		{#each zones as zone (zone.id)}
			{@const isSelected = selectedZoneId === zone.id}
			{@const materialName = getAssignedMaterial(zone.id)}
			<button
				type="button"
				class="w-full border-t border-border-light px-3 py-2.5 text-left transition-colors {isSelected
					? 'border-l-2 border-l-primary bg-primary-surface/10'
					: 'hover:bg-surface-alt'}"
				onclick={() => (selectedZoneId = zone.id)}
			>
				<div class="flex items-center gap-2">
					<span
						class="inline-flex rounded px-1.5 py-0.5 text-[10px] font-medium {ZONE_BADGE_COLORS[
							zone.zone_type
						] ?? 'bg-border text-text-secondary'}"
					>
						{zone.zone_type}
					</span>
					<span class="text-sm font-medium text-text truncate">
						{zone.label || zone.zone_type}
					</span>
				</div>
				<div class="mt-1 flex gap-3 text-xs text-text-secondary">
					<span>{zone.area_sqft.toFixed(0)} sq ft</span>
					<span>{zone.perimeter_ft.toFixed(0)} ft perim</span>
				</div>
				{#if materialName}
					<div class="mt-1 text-xs text-primary font-medium truncate">{materialName}</div>
				{:else}
					<div class="mt-1 text-xs text-text-tertiary italic">No material</div>
				{/if}
			</button>
		{/each}
	{/if}
</div>
