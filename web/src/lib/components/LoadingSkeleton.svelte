<script lang="ts">
	let { rows = 3, variant = 'card' }: { rows?: number; variant?: 'card' | 'row' | 'column' } =
		$props();
</script>

{#if variant === 'card'}
	<div class="space-y-3">
		{#each Array(rows) as _, i (i)}
			<div class="rounded-lg border border-border bg-surface p-5 animate-pulse">
				<div class="h-5 bg-border rounded w-1/3 mb-3"></div>
				<div class="h-4 bg-surface-hover rounded w-1/2"></div>
			</div>
		{/each}
	</div>
{:else if variant === 'row'}
	<div class="space-y-3 p-4">
		{#each Array(rows) as _, i (i)}
			<div class="animate-pulse flex items-center gap-3">
				<div class="h-4 bg-border rounded w-1/4"></div>
				<div class="h-4 bg-surface-hover rounded w-1/3"></div>
			</div>
		{/each}
	</div>
{:else if variant === 'column'}
	<div class="rounded-lg border border-border p-4">
		<div class="h-6 w-20 animate-pulse rounded bg-surface-hover mb-3"></div>
		<div class="h-10 w-28 animate-pulse rounded bg-surface-hover mb-4"></div>
		<div class="space-y-3">
			{#each Array(rows) as _, i (i)}
				<div class="h-16 animate-pulse rounded bg-surface-alt"></div>
			{/each}
		</div>
	</div>
{/if}
