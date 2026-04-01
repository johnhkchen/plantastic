<script lang="ts">
	import ErrorBanner from '$lib/components/ErrorBanner.svelte';
	import Viewer from '$lib/components/viewer/Viewer.svelte';

	let tappedZone = $state<string | null>(null);
	let activeTier = $state('good');
	let lightAngle = $state(30);
	let viewerError = $state<string | null>(null);
	let viewerRef = $state<ReturnType<typeof Viewer>>();

	// Tier scene URLs — all point to test scene until pt-scene generates real per-tier glTFs
	const tierUrls: Record<string, string> = {
		good: '/viewer/assets/models/test_scene.glb',
		better: '/viewer/assets/models/test_scene.glb',
		best: '/viewer/assets/models/test_scene.glb'
	};

	const tiers = ['good', 'better', 'best'] as const;

	function switchTier(tier: string) {
		activeTier = tier;
		viewerRef?.setTier(tier, tierUrls[tier]);
	}

	function onSliderInput(e: Event) {
		const target = e.target as HTMLInputElement;
		const degrees = Number(target.value);
		lightAngle = degrees;
		viewerRef?.setLightAngle(degrees);
	}

	/** Map light angle (0-360°) to approximate time of day string. */
	function degreesToTime(degrees: number): string {
		// 0° = 6:00 AM (sunrise), 90° = noon, 180° = 6:00 PM (sunset), 270° = midnight
		const hours = ((degrees / 360) * 24 + 6) % 24;
		const h = Math.floor(hours);
		const m = Math.floor((hours - h) * 60);
		const period = h >= 12 ? 'PM' : 'AM';
		const displayH = h === 0 ? 12 : h > 12 ? h - 12 : h;
		return `${displayH}:${m.toString().padStart(2, '0')} ${period}`;
	}
</script>

<div class="space-y-4">
	<h2 class="text-lg font-semibold text-text">3D Viewer</h2>

	{#if viewerError}
		<ErrorBanner message={viewerError} />
	{/if}

	<Viewer
		bind:this={viewerRef}
		sceneUrl={tierUrls[activeTier]}
		onZoneTapped={(id) => (tappedZone = id)}
		onError={(msg) => (viewerError = msg)}
		onLightAngleChanged={(degrees) => (lightAngle = degrees)}
		onTierChanged={(tier) => (activeTier = tier)}
	/>

	<!-- Tier Toggle -->
	<div class="flex items-center gap-2">
		<span class="text-sm font-medium text-text-secondary">Tier:</span>
		<div class="inline-flex rounded-lg border border-border bg-surface">
			{#each tiers as tier (tier)}
				<button
					class="min-h-[44px] px-4 py-2 text-sm font-medium first:rounded-l-lg last:rounded-r-lg {activeTier ===
					tier
						? 'bg-surface-invert text-text-invert'
						: 'text-text-secondary hover:bg-surface-alt'}"
					onclick={() => switchTier(tier)}
				>
					{tier.charAt(0).toUpperCase() + tier.slice(1)}
				</button>
			{/each}
		</div>
	</div>

	<!-- Sunlight Slider -->
	<div class="flex flex-wrap items-center gap-3">
		<span class="text-sm font-medium text-text-secondary">Sunlight:</span>
		<input
			type="range"
			min="0"
			max="360"
			value={lightAngle}
			oninput={onSliderInput}
			class="h-2 w-full max-w-48 cursor-pointer appearance-none rounded-lg bg-border"
		/>
		<span class="w-20 text-sm tabular-nums text-text-secondary">{degreesToTime(lightAngle)}</span>
	</div>

	{#if tappedZone}
		<div class="rounded-lg border border-border bg-surface p-4">
			<p class="text-sm text-text-secondary">Selected Zone</p>
			<p class="font-medium text-text">{tappedZone}</p>
		</div>
	{/if}
</div>
