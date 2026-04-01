<script lang="ts">
	import { page } from '$app/state';
	import { resolve } from '$app/paths';

	let { projectId }: { projectId: string } = $props();

	const tabs = [
		{ segment: 'editor', label: 'Editor' },
		{ segment: 'materials', label: 'Materials' },
		{ segment: 'quote', label: 'Quote' },
		{ segment: 'viewer', label: 'Viewer' },
		{ segment: 'export', label: 'Export' }
	] as const;

	function isActive(segment: string): boolean {
		return page.url.pathname.endsWith(`/${segment}`);
	}
</script>

<nav class="border-b border-border bg-surface">
	<div class="flex gap-0 overflow-x-auto px-4">
		{#each tabs as tab (tab.segment)}
			<a
				href={resolve(`/project/${projectId}/${tab.segment}`)}
				class="whitespace-nowrap border-b-2 px-4 py-3 text-sm font-medium transition-colors
					{isActive(tab.segment)
					? 'border-primary text-primary'
					: 'border-transparent text-text-secondary hover:border-border-strong hover:text-text'}"
			>
				{tab.label}
			</a>
		{/each}
	</div>
</nav>
