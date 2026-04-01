<script lang="ts">
	import type { Quote } from '$lib/api/quotes';

	let { quote, loading }: { quote: Quote | null; loading: boolean } = $props();

	const TIER_LABELS: Record<string, string> = {
		good: 'Good',
		better: 'Better',
		best: 'Best'
	};

	function formatUnit(unit: string): string {
		const labels: Record<string, string> = {
			sq_ft: 'sq ft',
			cu_yd: 'cu yd',
			linear_ft: 'lin ft',
			each: 'ea'
		};
		return labels[unit] ?? unit;
	}

	function fmt(value: string): string {
		const n = parseFloat(value);
		return isNaN(n) ? value : `$${n.toFixed(2)}`;
	}
</script>

<div class="flex h-full flex-col overflow-y-auto">
	<div class="px-3 py-2">
		<h3 class="text-xs font-medium uppercase text-text-secondary">
			Quote {quote ? `(${TIER_LABELS[quote.tier] ?? quote.tier})` : ''}
		</h3>
	</div>

	{#if loading}
		<div class="space-y-2 px-3 py-2">
			<div class="h-8 animate-pulse rounded bg-surface-hover"></div>
			<div class="h-4 animate-pulse rounded bg-surface-alt w-2/3"></div>
			<div class="h-4 animate-pulse rounded bg-surface-alt w-1/2"></div>
		</div>
	{:else if !quote || quote.line_items.length === 0}
		<div class="px-3 py-6 text-center">
			<p class="text-2xl font-bold text-text-muted">$0.00</p>
			<p class="mt-1 text-xs text-text-tertiary">Assign materials to see quote</p>
		</div>
	{:else}
		<!-- Total -->
		<div class="border-b border-border px-3 py-3">
			<p class="text-2xl font-bold text-text">{fmt(quote.total)}</p>
			{#if quote.tax}
				<p class="text-xs text-text-secondary">
					Subtotal: {fmt(quote.subtotal)} + Tax: {fmt(quote.tax)}
				</p>
			{/if}
		</div>

		<!-- Line items -->
		<div class="px-3 py-2">
			<h4 class="text-[11px] font-medium uppercase text-text-tertiary mb-1">Line Items</h4>
			{#each quote.line_items as li (li.zone_id + li.material_id)}
				<div class="border-t border-border-light py-1.5">
					<div class="flex items-center justify-between">
						<span class="text-xs text-text-secondary truncate max-w-[60%]">
							{li.zone_label ?? 'Zone'}
						</span>
						<span class="text-xs font-medium text-text font-mono">{fmt(li.line_total)}</span>
					</div>
					<div class="text-[11px] text-text-tertiary">
						{li.material_name} &middot; {parseFloat(li.quantity).toFixed(1)}
						{formatUnit(li.unit)} @ {fmt(li.unit_price)}
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>
