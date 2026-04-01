<script lang="ts">
	import TabNav from '$lib/components/TabNav.svelte';
	import { apiFetch } from '$lib/api';
	import type { Project } from '$lib/stores/project.svelte';
	import type { LayoutData } from './$types';

	let { data, children }: { data: LayoutData; children: import('svelte').Snippet } = $props();

	let projectId = $derived(data.id);
	let projectName = $state<string>('');

	$effect(() => {
		const id = projectId;
		projectName = `Project ${id.slice(0, 8)}...`;
		apiFetch<Project>(`/projects/${id}`)
			.then((project) => {
				projectName = project.client_name || project.address || 'Untitled Project';
			})
			.catch(() => {
				// Keep fallback name on error
			});
	});
</script>

<div class="mx-auto max-w-5xl">
	<div class="mb-4">
		<h1 class="font-display text-2xl font-bold text-text">{projectName}</h1>
	</div>

	<TabNav {projectId} />

	<div class="mt-6">
		{@render children()}
	</div>
</div>
