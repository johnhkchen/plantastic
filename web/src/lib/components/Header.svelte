<script lang="ts">
	import { session } from '$lib/stores/session.svelte';
</script>

<header class="flex h-14 items-center justify-between border-b border-gray-200 bg-white px-6">
	<h2 class="font-display text-sm font-semibold text-gray-700">{session.tenant}</h2>

	<div class="flex items-center gap-3">
		<div
			class="flex h-8 w-8 items-center justify-center rounded-full bg-brand-secondary text-xs font-medium text-white"
		>
			JC
		</div>
	</div>
</header>
