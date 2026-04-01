<script lang="ts">
	import { SvelteSet } from 'svelte/reactivity';
	import type { Quote, LineItem } from '$lib/api/quotes';

	let {
		quotes,
		loading
	}: {
		quotes: { good: Quote | null; better: Quote | null; best: Quote | null };
		loading: boolean;
	} = $props();

	const TIERS = ['good', 'better', 'best'] as const;
	const TIER_CONFIG: Record<string, { label: string; color: string; bg: string }> = {
		good: { label: 'Good', color: 'text-gray-700', bg: 'bg-gray-50' },
		better: { label: 'Better', color: 'text-brand-primary', bg: 'bg-brand-accent/10' },
		best: { label: 'Best', color: 'text-amber-800', bg: 'bg-amber-50' }
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
		if (isNaN(n)) return value;
		return n.toLocaleString('en-US', { style: 'currency', currency: 'USD' });
	}

	// Collect unique zone IDs across all tiers, preserving first-seen order
	let allZoneIds = $derived.by(() => {
		const ids: string[] = [];
		const seen = new SvelteSet<string>();
		for (const tier of TIERS) {
			for (const li of quotes[tier]?.line_items ?? []) {
				if (!seen.has(li.zone_id)) {
					seen.add(li.zone_id);
					ids.push(li.zone_id);
				}
			}
		}
		return ids;
	});

	// Build lookup: zoneId → tier → LineItem
	let zoneItemLookup = $derived.by(() => {
		const lookup: Record<string, Record<string, LineItem>> = {};
		for (const tier of TIERS) {
			for (const li of quotes[tier]?.line_items ?? []) {
				if (!lookup[li.zone_id]) lookup[li.zone_id] = {};
				lookup[li.zone_id][tier] = li;
			}
		}
		return lookup;
	});

	// Get zone label from first tier that has it
	function zoneLabel(zoneId: string): string {
		for (const tier of TIERS) {
			const li = zoneItemLookup[zoneId]?.[tier];
			if (li?.zone_label) return li.zone_label;
		}
		return 'Zone';
	}

	let isEmpty = $derived(
		!quotes.good?.line_items.length &&
			!quotes.better?.line_items.length &&
			!quotes.best?.line_items.length
	);
</script>

{#if loading}
	<div class="grid grid-cols-3 gap-4">
		{#each TIERS as tier (tier)}
			<div class="rounded-lg border border-gray-200 p-4">
				<div class="h-6 w-20 animate-pulse rounded bg-gray-100 mb-3"></div>
				<div class="h-10 w-28 animate-pulse rounded bg-gray-100 mb-4"></div>
				<div class="space-y-3">
					<div class="h-16 animate-pulse rounded bg-gray-50"></div>
					<div class="h-16 animate-pulse rounded bg-gray-50"></div>
				</div>
			</div>
		{/each}
	</div>
{:else if isEmpty}
	<div class="rounded-lg border border-gray-200 bg-white px-6 py-12 text-center">
		<p class="text-lg font-medium text-gray-400">No quotes to compare</p>
		<p class="mt-2 text-sm text-gray-400">
			<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- relative link, not a route -->
			Assign materials in the <a href="materials" class="text-brand-primary underline">Materials</a> tab
			to generate quotes.
		</p>
	</div>
{:else}
	<div class="grid grid-cols-3 gap-4">
		{#each TIERS as tier (tier)}
			{@const config = TIER_CONFIG[tier]}
			{@const quote = quotes[tier]}
			<div class="flex flex-col rounded-lg border border-gray-200 bg-white overflow-hidden">
				<!-- Tier header -->
				<div class="border-b border-gray-200 px-4 py-3 {config.bg}">
					<h3 class="text-sm font-medium uppercase tracking-wide {config.color}">
						{config.label}
					</h3>
					{#if quote && quote.line_items.length > 0}
						<p class="mt-1 text-2xl font-bold {config.color} font-mono">
							{fmt(quote.total)}
						</p>
						{#if quote.tax}
							<p class="text-xs text-gray-500">
								{fmt(quote.subtotal)} + {fmt(quote.tax)} tax
							</p>
						{/if}
					{:else}
						<p class="mt-1 text-2xl font-bold text-gray-300 font-mono">$0.00</p>
					{/if}
				</div>

				<!-- Zone rows -->
				<div class="flex-1 divide-y divide-gray-100">
					{#each allZoneIds as zoneId (zoneId)}
						{@const li = zoneItemLookup[zoneId]?.[tier]}
						<div class="px-4 py-3">
							<div class="text-xs font-medium text-gray-500 mb-1">
								{zoneLabel(zoneId)}
							</div>
							{#if li}
								<div class="text-sm text-gray-800">{li.material_name}</div>
								<div class="mt-0.5 text-xs text-gray-500">
									{parseFloat(li.quantity).toFixed(1)}
									{formatUnit(li.unit)} @ {fmt(li.unit_price)}
								</div>
								<div class="mt-0.5 text-sm font-medium text-gray-900 font-mono">
									{fmt(li.line_total)}
								</div>
							{:else}
								<div class="text-sm text-gray-300 italic">—</div>
							{/if}
						</div>
					{/each}
				</div>

				<!-- Footer total -->
				{#if quote && quote.line_items.length > 0}
					<div class="border-t border-gray-200 px-4 py-3 {config.bg}">
						<div class="flex items-center justify-between">
							<span class="text-sm font-medium text-gray-600">Total</span>
							<span class="text-lg font-bold {config.color} font-mono">{fmt(quote.total)}</span>
						</div>
					</div>
				{/if}
			</div>
		{/each}
	</div>
{/if}
