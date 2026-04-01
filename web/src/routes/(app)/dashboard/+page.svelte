<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { apiFetch } from '$lib/api';
	import EmptyState from '$lib/components/EmptyState.svelte';
	import ErrorBanner from '$lib/components/ErrorBanner.svelte';
	import LoadingSkeleton from '$lib/components/LoadingSkeleton.svelte';
	import type { Project } from '$lib/stores/project.svelte';
	import { projectStore } from '$lib/stores/project.svelte';
	import { STATUS_COLORS } from '$lib/styles/color-maps';
	import { onMount } from 'svelte';

	let loading = $state(true);
	let error = $state<string | null>(null);
	let showCreateModal = $state(false);

	// Create form fields
	let newAddress = $state('');
	let newClientName = $state('');
	let newClientEmail = $state('');
	let creating = $state(false);

	onMount(() => {
		loadProjects();
	});

	async function loadProjects() {
		loading = true;
		error = null;
		try {
			projectStore.projects = await apiFetch<Project[]>('/projects');
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load projects';
		} finally {
			loading = false;
		}
	}

	async function createProject() {
		creating = true;
		try {
			const project = await apiFetch<Project>('/projects', {
				method: 'POST',
				body: {
					address: newAddress || undefined,
					client_name: newClientName || undefined,
					client_email: newClientEmail || undefined
				}
			});
			projectStore.projects = [...projectStore.projects, project];
			showCreateModal = false;
			newAddress = '';
			newClientName = '';
			newClientEmail = '';
			goto(resolve(`/project/${project.id}`));
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to create project';
		} finally {
			creating = false;
		}
	}

	function projectDisplayName(p: Project): string {
		return p.client_name || p.address || 'Untitled Project';
	}

	function formatDate(iso: string): string {
		return new Date(iso).toLocaleDateString('en-US', {
			month: 'short',
			day: 'numeric',
			year: 'numeric'
		});
	}
</script>

<div class="mx-auto max-w-4xl">
	<div class="mb-6 flex items-center justify-between">
		<h1 class="font-display text-2xl font-bold text-text">Projects</h1>
		<button
			onclick={() => (showCreateModal = true)}
			class="rounded-md bg-primary px-4 py-2 text-sm font-medium text-text-invert shadow-sm hover:bg-primary-light transition-colors"
		>
			New Project
		</button>
	</div>

	{#if error}
		<ErrorBanner message={error} onretry={loadProjects} />
	{/if}

	{#if loading}
		<LoadingSkeleton variant="card" />
	{:else if projectStore.projects.length === 0 && !error}
		<EmptyState icon="📋" message="No projects yet" submessage="Create your first project to get started">
			<button
				onclick={() => (showCreateModal = true)}
				class="rounded-md bg-primary px-4 py-2 text-sm font-medium text-text-invert shadow-sm hover:bg-primary-light transition-colors"
			>
				New Project
			</button>
		</EmptyState>
	{:else}
		<div class="space-y-3">
			{#each projectStore.projects as project (project.id)}
				<a
					href={resolve(`/project/${project.id}`)}
					class="block rounded-lg border border-border bg-surface p-5 hover:border-primary hover:shadow-sm transition-all"
				>
					<div class="flex items-start justify-between">
						<div>
							<h2 class="font-medium text-text">{projectDisplayName(project)}</h2>
							{#if project.address && project.client_name}
								<p class="mt-1 text-sm text-text-secondary">{project.address}</p>
							{/if}
						</div>
						<span
							class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium {STATUS_COLORS[
								project.status
							] ?? 'bg-surface-hover text-text-secondary'}"
						>
							{project.status}
						</span>
					</div>
					<p class="mt-2 text-xs text-text-tertiary">{formatDate(project.created_at)}</p>
				</a>
			{/each}
		</div>
	{/if}
</div>

<!-- Create Project Modal -->
{#if showCreateModal}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
		<div class="w-full max-w-md rounded-lg bg-surface p-6 shadow-xl">
			<h2 class="text-lg font-semibold text-text mb-4">New Project</h2>
			<form
				onsubmit={(e) => {
					e.preventDefault();
					createProject();
				}}
			>
				<div class="space-y-4">
					<div>
						<label for="address" class="block text-sm font-medium text-text-secondary">Address</label>
						<input
							id="address"
							type="text"
							bind:value={newAddress}
							placeholder="123 Main St, City, State"
							class="mt-1 block w-full rounded-md border border-border-strong px-3 py-2 text-sm shadow-sm focus:border-primary focus:ring-1 focus:ring-primary"
						/>
					</div>
					<div>
						<label for="client_name" class="block text-sm font-medium text-text-secondary"
							>Client Name</label
						>
						<input
							id="client_name"
							type="text"
							bind:value={newClientName}
							placeholder="Jane Smith"
							class="mt-1 block w-full rounded-md border border-border-strong px-3 py-2 text-sm shadow-sm focus:border-primary focus:ring-1 focus:ring-primary"
						/>
					</div>
					<div>
						<label for="client_email" class="block text-sm font-medium text-text-secondary"
							>Client Email</label
						>
						<input
							id="client_email"
							type="email"
							bind:value={newClientEmail}
							placeholder="jane@example.com"
							class="mt-1 block w-full rounded-md border border-border-strong px-3 py-2 text-sm shadow-sm focus:border-primary focus:ring-1 focus:ring-primary"
						/>
					</div>
				</div>
				<div class="mt-6 flex justify-end gap-3">
					<button
						type="button"
						onclick={() => (showCreateModal = false)}
						class="rounded-md border border-border-strong px-4 py-2 text-sm font-medium text-text-secondary hover:bg-surface-alt"
					>
						Cancel
					</button>
					<button
						type="submit"
						disabled={creating}
						class="rounded-md bg-primary px-4 py-2 text-sm font-medium text-text-invert shadow-sm hover:bg-primary-light disabled:opacity-50 transition-colors"
					>
						{creating ? 'Creating...' : 'Create Project'}
					</button>
				</div>
			</form>
		</div>
	</div>
{/if}
