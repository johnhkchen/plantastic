<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { apiFetch } from '$lib/api';
	import type { Project } from '$lib/stores/project.svelte';
	import { projectStore } from '$lib/stores/project.svelte';
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

	const statusColors: Record<string, string> = {
		draft: 'bg-gray-100 text-gray-700',
		quoted: 'bg-blue-100 text-blue-700',
		approved: 'bg-green-100 text-green-700',
		complete: 'bg-purple-100 text-purple-700'
	};
</script>

<div class="mx-auto max-w-4xl">
	<div class="mb-6 flex items-center justify-between">
		<h1 class="font-display text-2xl font-bold text-gray-900">Projects</h1>
		<button
			onclick={() => (showCreateModal = true)}
			class="rounded-md bg-brand-primary px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-brand-secondary transition-colors"
		>
			New Project
		</button>
	</div>

	{#if error}
		<div
			class="mb-4 rounded-md bg-red-50 border border-red-200 p-4 flex items-center justify-between"
		>
			<p class="text-sm text-red-700">{error}</p>
			<button
				onclick={loadProjects}
				class="text-sm font-medium text-red-700 hover:text-red-800 underline"
			>
				Retry
			</button>
		</div>
	{/if}

	{#if loading}
		<div class="space-y-3">
			{#each [1, 2, 3] as n (n)}
				<div class="rounded-lg border border-gray-200 bg-white p-5 animate-pulse">
					<div class="h-5 bg-gray-200 rounded w-1/3 mb-3"></div>
					<div class="h-4 bg-gray-100 rounded w-1/2"></div>
				</div>
			{/each}
		</div>
	{:else if projectStore.projects.length === 0 && !error}
		<div class="rounded-lg border border-gray-200 bg-white p-12 text-center">
			<p class="text-gray-500">No projects yet. Create your first project to get started.</p>
		</div>
	{:else}
		<div class="space-y-3">
			{#each projectStore.projects as project (project.id)}
				<a
					href={resolve(`/project/${project.id}`)}
					class="block rounded-lg border border-gray-200 bg-white p-5 hover:border-brand-primary hover:shadow-sm transition-all"
				>
					<div class="flex items-start justify-between">
						<div>
							<h2 class="font-medium text-gray-900">{projectDisplayName(project)}</h2>
							{#if project.address && project.client_name}
								<p class="mt-1 text-sm text-gray-500">{project.address}</p>
							{/if}
						</div>
						<span
							class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium {statusColors[
								project.status
							] ?? 'bg-gray-100 text-gray-700'}"
						>
							{project.status}
						</span>
					</div>
					<p class="mt-2 text-xs text-gray-400">{formatDate(project.created_at)}</p>
				</a>
			{/each}
		</div>
	{/if}
</div>

<!-- Create Project Modal -->
{#if showCreateModal}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
		<div class="w-full max-w-md rounded-lg bg-white p-6 shadow-xl">
			<h2 class="text-lg font-semibold text-gray-900 mb-4">New Project</h2>
			<form
				onsubmit={(e) => {
					e.preventDefault();
					createProject();
				}}
			>
				<div class="space-y-4">
					<div>
						<label for="address" class="block text-sm font-medium text-gray-700">Address</label>
						<input
							id="address"
							type="text"
							bind:value={newAddress}
							placeholder="123 Main St, City, State"
							class="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 text-sm shadow-sm focus:border-brand-primary focus:ring-1 focus:ring-brand-primary"
						/>
					</div>
					<div>
						<label for="client_name" class="block text-sm font-medium text-gray-700"
							>Client Name</label
						>
						<input
							id="client_name"
							type="text"
							bind:value={newClientName}
							placeholder="Jane Smith"
							class="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 text-sm shadow-sm focus:border-brand-primary focus:ring-1 focus:ring-brand-primary"
						/>
					</div>
					<div>
						<label for="client_email" class="block text-sm font-medium text-gray-700"
							>Client Email</label
						>
						<input
							id="client_email"
							type="email"
							bind:value={newClientEmail}
							placeholder="jane@example.com"
							class="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 text-sm shadow-sm focus:border-brand-primary focus:ring-1 focus:ring-brand-primary"
						/>
					</div>
				</div>
				<div class="mt-6 flex justify-end gap-3">
					<button
						type="button"
						onclick={() => (showCreateModal = false)}
						class="rounded-md border border-gray-300 px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50"
					>
						Cancel
					</button>
					<button
						type="submit"
						disabled={creating}
						class="rounded-md bg-brand-primary px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-brand-secondary disabled:opacity-50 transition-colors"
					>
						{creating ? 'Creating...' : 'Create Project'}
					</button>
				</div>
			</form>
		</div>
	</div>
{/if}
