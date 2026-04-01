<script lang="ts">
	import { friendlyError } from '$lib/utils/errors';

	let { message, onretry }: { message: string; onretry?: () => void } = $props();

	let displayMessage = $derived(friendlyError(message));
</script>

<div class="mb-4 rounded-md bg-error-surface border border-error/20 p-4 flex items-center justify-between">
	<p class="text-sm text-error-text">{displayMessage}</p>
	{#if onretry}
		<button
			onclick={onretry}
			class="ml-4 shrink-0 text-sm font-medium text-error-text hover:text-error underline"
		>
			Retry
		</button>
	{/if}
</div>
