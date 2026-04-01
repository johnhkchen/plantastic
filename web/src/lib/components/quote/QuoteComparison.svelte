<script lang="ts">
	import { SvelteSet } from 'svelte/reactivity';
	import type { Quote, LineItem } from '$lib/api/quotes';
	import EmptyState from '$lib/components/EmptyState.svelte';
	import TierTabs from '$lib/components/assignment/TierTabs.svelte';
	import { TIER_CONFIG } from '$lib/styles/color-maps';

	let {
		quotes,
		loading
	}: {
		quotes: { good: Quote | null; better: Quote | null; best: Quote | null };
		loading: boolean;
	} = $props();

	const TIERS = ['good', 'better', 'best'] as const;

	let activeTier = $state<string>('good');

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

{#snippet tierCard(tier: typeof TIERS[number])}
	{@const config = TIER_CONFIG[tier]}
	{@const quote = quotes[tier]}
	<div class="flex flex-col rounded-lg border border-border bg-surface overflow-hidden">
		<!-- Tier header -->
		<div class="border-b border-border px-4 py-3 {config.bg}">
			<h3 class="text-sm font-medium uppercase tracking-wide {config.color}">
				{config.label}
			</h3>
			{#if quote && quote.line_items.length > 0}
				<p class="mt-1 text-2xl font-bold {config.color} font-mono">
					{fmt(quote.total)}
				</p>
				{#if quote.tax}
					<p class="text-xs text-text-secondary">
						{fmt(quote.subtotal)} + {fmt(quote.tax)} tax
					</p>
				{/if}
			{:else}
				<p class="mt-1 text-2xl font-bold text-text-muted font-mono">$0.00</p>
			{/if}
		</div>

		<!-- Zone rows -->
		<div class="flex-1 divide-y divide-border-light">
			{#each allZoneIds as zoneId (zoneId)}
				{@const li = zoneItemLookup[zoneId]?.[tier]}
				<div class="px-4 py-3">
					<div class="text-xs font-medium text-text-secondary mb-1">
						{zoneLabel(zoneId)}
					</div>
					{#if li}
						<div class="text-sm text-text">{li.material_name}</div>
						<div class="mt-0.5 text-xs text-text-secondary">
							{parseFloat(li.quantity).toFixed(1)}
							{formatUnit(li.unit)} @ {fmt(li.unit_price)}
						</div>
						<div class="mt-0.5 text-sm font-medium text-text font-mono">
							{fmt(li.line_total)}
						</div>
					{:else}
						<div class="text-sm text-text-muted italic">—</div>
					{/if}
				</div>
			{/each}
		</div>

		<!-- Footer total -->
		{#if quote && quote.line_items.length > 0}
			<div class="border-t border-border px-4 py-3 {config.bg}">
				<div class="flex items-center justify-between">
					<span class="text-sm font-medium text-text-secondary">Total</span>
					<span class="text-lg font-bold {config.color} font-mono">{fmt(quote.total)}</span>
				</div>
			</div>
		{/if}
	</div>
{/snippet}

{#if loading}
	<!-- Loading skeleton: 1 col mobile, 3 col desktop -->
	<div class="grid grid-cols-1 lg:grid-cols-3 gap-4">
		{#each TIERS as tier (tier)}
			<div class="rounded-lg border border-border p-4 first:block hidden lg:block">
				<div class="h-6 w-20 animate-pulse rounded bg-surface-hover mb-3"></div>
				<div class="h-10 w-28 animate-pulse rounded bg-surface-hover mb-4"></div>
				<div class="space-y-3">
					<div class="h-16 animate-pulse rounded bg-surface-alt"></div>
					<div class="h-16 animate-pulse rounded bg-surface-alt"></div>
				</div>
			</div>
		{/each}
	</div>
{:else if isEmpty}
	<EmptyState icon="💰" message="No quotes to compare" submessage="Assign materials to zones to generate quotes">
		<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- relative link, not a route -->
		<a href="materials" class="text-sm font-medium text-primary hover:text-primary-light underline">
			Go to Materials
		</a>
	</EmptyState>
{:else}
	<!-- Desktop: 3-column grid -->
	<div class="hidden lg:grid lg:grid-cols-3 lg:gap-4">
		{#each TIERS as tier (tier)}
			{@render tierCard(tier)}
		{/each}
	</div>

	<!-- Tablet: horizontal scroll with snap -->
	<div class="hidden md:flex lg:hidden overflow-x-auto snap-x snap-mandatory gap-4 pb-2 -mx-4 px-4">
		{#each TIERS as tier (tier)}
			<div class="min-w-[300px] flex-shrink-0 snap-center">
				{@render tierCard(tier)}
			</div>
		{/each}
	</div>

	<!-- Mobile: tab switcher + single card -->
	<div class="md:hidden">
		<div class="mb-4">
			<TierTabs bind:activeTier />
		</div>
		{#each TIERS as tier (tier)}
			{#if tier === activeTier}
				{@render tierCard(tier)}
			{/if}
		{/each}
	</div>
{/if}
