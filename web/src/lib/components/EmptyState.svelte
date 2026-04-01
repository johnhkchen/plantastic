<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		icon,
		message,
		submessage,
		children
	}: {
		icon: string;
		message: string;
		submessage?: string;
		children?: Snippet;
	} = $props();
</script>

<div class="rounded-lg border border-border bg-surface px-6 py-12 text-center">
	<div class="mb-3 text-4xl">{icon}</div>
	<p class="text-lg font-medium text-text-secondary">{message}</p>
	{#if submessage}
		<p class="mt-1 text-sm text-text-tertiary">{submessage}</p>
	{/if}
	{#if children}
		<div class="mt-4">
			{@render children()}
		</div>
	{/if}
</div>
