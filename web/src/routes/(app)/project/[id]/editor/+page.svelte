<script lang="ts">
	import ZoneEditor from '$lib/components/zone-editor/ZoneEditor.svelte';
	import type { EditorZone } from '$lib/components/zone-editor/types';
	import type { ApiZone } from '$lib/api/types';
	import { apiZoneToEditorZone } from '$lib/api/types';
	import { fetchZones, saveZones } from '$lib/api/zones';
	import type { LayoutData } from '../$types';

	let { data }: { data: LayoutData } = $props();
	let projectId = $derived(data.id);

	let zones = $state<EditorZone[]>([]);
	let apiZones = $state<ApiZone[]>([]);
	let loading = $state(true);
	let saving = $state(false);
	let error = $state<string | null>(null);

	// Load zones on mount / project change
	$effect(() => {
		const id = projectId;
		loading = true;
		error = null;
		fetchZones(id)
			.then((fetched) => {
				apiZones = fetched;
				zones = fetched.map(apiZoneToEditorZone);
			})
			.catch((e) => {
				error = e instanceof Error ? e.message : 'Failed to load zones';
			})
			.finally(() => {
				loading = false;
			});
	});

	// Debounced auto-save when zones change
	let saveTimer: ReturnType<typeof setTimeout> | null = null;
	let initialized = false;

	$effect(() => {
		// Track zone changes (touch all fields to establish dependency)
		const _trigger = zones.map((z) => [z.id, z.vertices, z.zoneType, z.label]);
		void _trigger;

		if (!initialized) {
			initialized = true;
			return;
		}

		if (saveTimer) clearTimeout(saveTimer);
		saveTimer = setTimeout(() => {
			if (zones.length === 0 && apiZones.length === 0) return;
			saving = true;
			saveZones(projectId, zones)
				.then((saved) => {
					apiZones = saved;
					// Re-sync IDs from server (bulk PUT creates new IDs)
					zones = saved.map(apiZoneToEditorZone);
					error = null;
				})
				.catch((e) => {
					error = e instanceof Error ? e.message : 'Failed to save';
				})
				.finally(() => {
					saving = false;
				});
		}, 1500);

		return () => {
			if (saveTimer) clearTimeout(saveTimer);
		};
	});

	// Measurement lookup by zone ID
	function getMeasurements(zoneId: string): { area: number; perimeter: number } | null {
		const az = apiZones.find((z) => z.id === zoneId);
		if (!az) return null;
		return { area: az.area_sqft, perimeter: az.perimeter_ft };
	}
</script>

<div class="flex h-full flex-col">
	<div class="flex items-center justify-between border-b border-gray-200 px-4 py-3">
		<h2 class="text-lg font-semibold text-gray-900">Zone Editor</h2>
		<div class="flex items-center gap-2">
			{#if saving}
				<span class="text-xs text-gray-400">Saving...</span>
			{:else if error}
				<span class="text-xs text-red-500">{error}</span>
			{:else if !loading && apiZones.length > 0}
				<span class="text-xs text-green-600">Saved</span>
			{/if}
		</div>
	</div>

	{#if loading}
		<div class="flex flex-1 items-center justify-center">
			<span class="text-sm text-gray-400">Loading zones...</span>
		</div>
	{:else}
		<div class="flex min-h-0 flex-1">
			<!-- Canvas -->
			<div class="min-w-0 flex-1">
				<ZoneEditor bind:zones />
			</div>

			<!-- Zone info panel -->
			{#if zones.length > 0}
				<div class="w-64 overflow-y-auto border-l border-gray-200 bg-white">
					<div class="px-3 py-2">
						<h3 class="text-xs font-medium uppercase text-gray-500">Zones</h3>
					</div>
					{#each zones as zone (zone.id)}
						{@const m = getMeasurements(zone.id)}
						<div class="border-t border-gray-100 px-3 py-2">
							<div class="text-sm font-medium text-gray-800">
								{zone.label || zone.zoneType}
							</div>
							<div class="text-xs text-gray-500 capitalize">{zone.zoneType}</div>
							{#if m}
								<div class="mt-1 flex gap-3">
									<span class="text-xs text-gray-600">
										<span class="font-medium">{m.area.toFixed(1)}</span> sq ft
									</span>
									<span class="text-xs text-gray-600">
										<span class="font-medium">{m.perimeter.toFixed(1)}</span> ft
									</span>
								</div>
							{/if}
						</div>
					{/each}
				</div>
			{/if}
		</div>
	{/if}
</div>
