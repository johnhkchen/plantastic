<script lang="ts">
	import { apiFetch } from '$lib/api';
	import type { ApiZone } from '$lib/api/types';
	import type { Material } from '$lib/stores/project.svelte';
	import { fetchTiers, saveTierAssignments } from '$lib/api/tiers';
	import type { AssignmentResponse, AssignmentInput } from '$lib/api/tiers';
	import { fetchQuote } from '$lib/api/quotes';
	import type { Quote } from '$lib/api/quotes';
	import ZoneList from '$lib/components/assignment/ZoneList.svelte';
	import MaterialPicker from '$lib/components/assignment/MaterialPicker.svelte';
	import TierTabs from '$lib/components/assignment/TierTabs.svelte';
	import QuoteSummary from '$lib/components/assignment/QuoteSummary.svelte';
	import type { LayoutData } from '../$types';

	let { data }: { data: LayoutData } = $props();
	let projectId = $derived(data.id);

	// Data state
	let zones = $state<ApiZone[]>([]);
	let materials = $state<Material[]>([]);
	let tierData = $state<Record<string, AssignmentResponse[]>>({
		good: [],
		better: [],
		best: []
	});

	// UI state
	let activeTier = $state<string>('good');
	let selectedZoneId = $state<string | null>(null);
	let quote = $state<Quote | null>(null);

	// Loading / error
	let loading = $state(true);
	let saving = $state(false);
	let quoteLoading = $state(false);
	let error = $state<string | null>(null);

	// Current tier assignments
	let activeAssignments = $derived(tierData[activeTier] ?? []);

	// Material assigned to selected zone in active tier
	let currentMaterialId = $derived.by(() => {
		if (!selectedZoneId) return null;
		const a = activeAssignments.find((a) => a.zone_id === selectedZoneId);
		return a?.material_id ?? null;
	});

	// Load all data on mount / project change
	$effect(() => {
		const id = projectId;
		loading = true;
		error = null;
		quote = null;

		Promise.all([
			apiFetch<ApiZone[]>(`/projects/${id}/zones`),
			apiFetch<Material[]>('/materials'),
			fetchTiers(id)
		])
			.then(([z, m, t]) => {
				zones = z;
				materials = m;
				const td: Record<string, AssignmentResponse[]> = { good: [], better: [], best: [] };
				for (const tier of t) {
					td[tier.tier] = tier.assignments;
				}
				tierData = td;
			})
			.catch((e) => {
				error = e instanceof Error ? e.message : 'Failed to load data';
			})
			.finally(() => {
				loading = false;
			});
	});

	// Fetch quote when tier changes or after initial load
	$effect(() => {
		const tier = activeTier;
		const id = projectId;
		if (loading) return;

		quoteLoading = true;
		fetchQuote(id, tier)
			.then((q) => {
				quote = q;
			})
			.catch(() => {
				quote = null;
			})
			.finally(() => {
				quoteLoading = false;
			});
	});

	// Debounced save
	let saveTimer: ReturnType<typeof setTimeout> | null = null;

	function handleAssign(materialId: string) {
		if (!selectedZoneId) return;

		// Toggle: if clicking the already-assigned material, unassign
		const isUnassign = currentMaterialId === materialId;

		// Update local state immediately (optimistic)
		const tier = activeTier;
		const current = [...(tierData[tier] ?? [])];

		// Remove existing assignment for this zone
		const filtered = current.filter((a) => a.zone_id !== selectedZoneId);

		if (!isUnassign) {
			filtered.push({ zone_id: selectedZoneId, material_id: materialId, overrides: null });
		}

		tierData = { ...tierData, [tier]: filtered };

		// Debounced save
		if (saveTimer) clearTimeout(saveTimer);
		saveTimer = setTimeout(() => {
			persistAssignments(tier);
		}, 800);
	}

	async function persistAssignments(tier: string) {
		saving = true;
		error = null;
		try {
			const assignments: AssignmentInput[] = (tierData[tier] ?? []).map((a) => ({
				zone_id: a.zone_id,
				material_id: a.material_id,
				overrides: a.overrides
			}));
			await saveTierAssignments(projectId, tier, assignments);

			// Refresh quote after save
			quoteLoading = true;
			const q = await fetchQuote(projectId, tier);
			// Only update if we're still on the same tier
			if (activeTier === tier) {
				quote = q;
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to save assignments';
		} finally {
			saving = false;
			quoteLoading = false;
		}
	}
</script>

<div class="flex h-[calc(100vh-12rem)] flex-col">
	<!-- Header -->
	<div class="flex items-center justify-between pb-3">
		<h2 class="text-lg font-semibold text-gray-900">Material Assignments</h2>
		<div class="flex items-center gap-2">
			{#if saving}
				<span class="text-xs text-gray-400">Saving...</span>
			{:else if error}
				<span class="text-xs text-red-500">{error}</span>
			{/if}
		</div>
	</div>

	{#if loading}
		<div class="flex flex-1 items-center justify-center">
			<span class="text-sm text-gray-400">Loading...</span>
		</div>
	{:else}
		<!-- Three-column layout -->
		<div
			class="flex min-h-0 flex-1 gap-0 overflow-hidden rounded-lg border border-gray-200 bg-white"
		>
			<!-- Left: Zone List -->
			<div class="w-52 shrink-0 border-r border-gray-200">
				<ZoneList {zones} bind:selectedZoneId assignments={activeAssignments} {materials} />
			</div>

			<!-- Center: Tier Tabs + Material Picker -->
			<div class="flex min-w-0 flex-1 flex-col">
				<TierTabs bind:activeTier />
				<div class="min-h-0 flex-1">
					<MaterialPicker
						{materials}
						{selectedZoneId}
						{currentMaterialId}
						onAssign={handleAssign}
					/>
				</div>
			</div>

			<!-- Right: Quote Summary -->
			<div class="w-56 shrink-0 border-l border-gray-200">
				<QuoteSummary {quote} loading={quoteLoading} />
			</div>
		</div>
	{/if}
</div>
