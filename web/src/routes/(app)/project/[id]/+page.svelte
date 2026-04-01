<script lang="ts">
	import { apiFetch } from '$lib/api';
	import type { Project, ProjectBaseline } from '$lib/stores/project.svelte';
	import { projectStore } from '$lib/stores/project.svelte';
	import { STATUS_COLORS } from '$lib/styles/color-maps';
	import type { LayoutData } from './$types';
	import { onMount } from 'svelte';

	let { data }: { data: LayoutData } = $props();

	let loading = $state(true);
	let error = $state<string | null>(null);
	let project = $state<Project | null>(null);
	let zoneCount = $state(0);
	let baseline = $derived<ProjectBaseline | null>(project?.baseline ?? null);

	onMount(() => {
		loadProject();
	});

	async function loadProject() {
		loading = true;
		error = null;
		try {
			const [proj, zones] = await Promise.all([
				apiFetch<Project>(`/projects/${data.id}`),
				apiFetch<unknown[]>(`/projects/${data.id}/zones`)
			]);
			project = proj;
			projectStore.current = proj;
			zoneCount = zones.length;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load project';
		} finally {
			loading = false;
		}
	}

	function formatDate(iso: string): string {
		return new Date(iso).toLocaleDateString('en-US', {
			month: 'short',
			day: 'numeric',
			year: 'numeric',
			hour: 'numeric',
			minute: '2-digit'
		});
	}
</script>

{#if loading}
	<div class="space-y-4 animate-pulse">
		<div class="h-6 bg-border rounded w-1/3"></div>
		<div class="h-4 bg-surface-hover rounded w-1/2"></div>
		<div class="grid grid-cols-3 gap-4 mt-6">
			{#each [1, 2, 3] as n (n)}
				<div class="rounded-lg border border-border bg-surface p-5">
					<div class="h-4 bg-border rounded w-1/2 mb-2"></div>
					<div class="h-6 bg-surface-hover rounded w-2/3"></div>
				</div>
			{/each}
		</div>
	</div>
{:else if error}
	<div class="rounded-md bg-error-surface border border-error/20 p-4 flex items-center justify-between">
		<p class="text-sm text-error-text">{error}</p>
		<button
			onclick={loadProject}
			class="text-sm font-medium text-error-text hover:text-error underline"
		>
			Retry
		</button>
	</div>
{:else if project}
	<div>
		<div class="mb-6">
			<div class="flex items-center gap-3">
				<h2 class="text-lg font-semibold text-text">
					{project.client_name || 'Untitled Project'}
				</h2>
				<span
					class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium {STATUS_COLORS[
						project.status
					] ?? 'bg-surface-hover text-text-secondary'}"
				>
					{project.status}
				</span>
			</div>
			{#if project.address}
				<p class="mt-1 text-sm text-text-secondary">{project.address}</p>
			{/if}
		</div>

		<div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
			<div class="rounded-lg border border-border bg-surface p-5">
				<p class="text-xs font-medium text-text-secondary uppercase tracking-wider">Zones</p>
				<p class="mt-1 text-2xl font-semibold text-text">{zoneCount}</p>
			</div>
			<div class="rounded-lg border border-border bg-surface p-5">
				<p class="text-xs font-medium text-text-secondary uppercase tracking-wider">Created</p>
				<p class="mt-1 text-sm font-medium text-text">{formatDate(project.created_at)}</p>
			</div>
			<div class="rounded-lg border border-border bg-surface p-5">
				<p class="text-xs font-medium text-text-secondary uppercase tracking-wider">Client Email</p>
				<p class="mt-1 text-sm font-medium text-text">{project.client_email || '—'}</p>
			</div>
		</div>

		{#if baseline}
			<div class="mt-8">
				<h3 class="text-sm font-semibold text-text uppercase tracking-wider mb-4">
					Site Baseline
				</h3>
				<div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
					<div class="rounded-lg border border-border bg-surface p-5">
						<p class="text-xs font-medium text-text-secondary uppercase tracking-wider">Lot Area</p>
						<p class="mt-1 text-2xl font-semibold text-text">
							{Math.round(baseline.lot_boundary.area_sqft).toLocaleString()} sqft
						</p>
					</div>
					<div class="rounded-lg border border-border bg-surface p-5">
						<p class="text-xs font-medium text-text-secondary uppercase tracking-wider">Trees Detected</p>
						<p class="mt-1 text-2xl font-semibold text-text">{baseline.trees.length}</p>
					</div>
					<div class="rounded-lg border border-border bg-surface p-5">
						<p class="text-xs font-medium text-text-secondary uppercase tracking-wider">Sun Grid</p>
						<p class="mt-1 text-2xl font-semibold text-text">
							{baseline.sun_grid.width} &times; {baseline.sun_grid.height}
						</p>
						<p class="text-xs text-text-secondary">{baseline.sun_grid.values.length} cells</p>
					</div>
				</div>

				{#if baseline.trees.length > 0}
					<div class="mt-4 rounded-lg border border-border bg-surface overflow-hidden">
						<table class="min-w-full divide-y divide-border">
							<thead class="bg-surface-alt">
								<tr>
									<th class="px-4 py-2 text-left text-xs font-medium text-text-secondary uppercase">#</th>
									<th class="px-4 py-2 text-left text-xs font-medium text-text-secondary uppercase"
										>Height (ft)</th
									>
									<th class="px-4 py-2 text-left text-xs font-medium text-text-secondary uppercase"
										>Spread (ft)</th
									>
									<th class="px-4 py-2 text-left text-xs font-medium text-text-secondary uppercase"
										>Confidence</th
									>
								</tr>
							</thead>
							<tbody class="divide-y divide-border">
								{#each baseline.trees as tree, i (i)}
									<tr>
										<td class="px-4 py-2 text-sm text-text-secondary">{i + 1}</td>
										<td class="px-4 py-2 text-sm text-text">{tree.height_ft.toFixed(1)}</td>
										<td class="px-4 py-2 text-sm text-text">{tree.spread_ft.toFixed(1)}</td>
										<td class="px-4 py-2 text-sm text-text"
											>{(tree.confidence * 100).toFixed(0)}%</td
										>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/if}
			</div>
		{/if}
	</div>
{/if}
