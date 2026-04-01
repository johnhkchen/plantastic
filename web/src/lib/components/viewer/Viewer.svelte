<script lang="ts">
	import { isViewerMessage } from './types';
	import type { ViewerInboundMessage } from './types';

	let {
		sceneUrl,
		onZoneTapped,
		onReady,
		onError,
		onLightAngleChanged,
		onTierChanged
	}: {
		sceneUrl: string;
		onZoneTapped?: (zoneId: string) => void;
		onReady?: () => void;
		onError?: (message: string) => void;
		onLightAngleChanged?: (degrees: number) => void;
		onTierChanged?: (tier: string) => void;
	} = $props();

	let iframeEl: HTMLIFrameElement | undefined = $state();
	let ready = $state(false);

	function sendCommand(msg: ViewerInboundMessage) {
		if (iframeEl?.contentWindow) {
			iframeEl.contentWindow.postMessage(msg, '*');
		}
	}

	export function setTier(tier: string, url: string) {
		sendCommand({ type: 'setTier', tier, url });
	}

	export function setLightAngle(degrees: number) {
		sendCommand({ type: 'setLightAngle', degrees });
	}

	$effect(() => {
		function handleMessage(event: MessageEvent) {
			if (!isViewerMessage(event)) return;

			const msg = event.data;
			switch (msg.type) {
				case 'ready':
					if (!ready) {
						ready = true;
						sendCommand({ type: 'loadScene', url: sceneUrl });
						onReady?.();
					}
					break;
				case 'zoneTapped':
					onZoneTapped?.(msg.zoneId);
					break;
				case 'error':
					onError?.(msg.message);
					break;
				case 'lightAngleChanged':
					onLightAngleChanged?.(msg.degrees);
					break;
				case 'tierChanged':
					onTierChanged?.(msg.tier);
					break;
			}
		}

		window.addEventListener('message', handleMessage);
		return () => window.removeEventListener('message', handleMessage);
	});
</script>

<div
	class="relative aspect-video w-full overflow-hidden rounded-lg bg-gray-900"
	data-viewer-ready={ready || undefined}
>
	<iframe
		bind:this={iframeEl}
		src="/viewer/index.html"
		class="h-full w-full border-0"
		title="3D Viewer"
		allow="autoplay"
	></iframe>
	{#if !ready}
		<div
			class="absolute inset-0 flex items-center justify-center bg-gray-900/80 text-sm text-gray-400"
		>
			Loading viewer...
		</div>
	{/if}
</div>
