/** Messages sent from the SvelteKit host to the Bevy viewer iframe. */
export type ViewerInboundMessage =
	| { type: 'loadScene'; url: string }
	| { type: 'setTier'; tier: string; url: string }
	| { type: 'setLightAngle'; degrees: number };

/** Messages sent from the Bevy viewer iframe to the SvelteKit host. */
export type ViewerOutboundMessage =
	| { type: 'ready' }
	| { type: 'error'; message: string }
	| { type: 'zoneTapped'; zoneId: string }
	| { type: 'lightAngleChanged'; degrees: number }
	| { type: 'tierChanged'; tier: string };

/** Type guard: check if a MessageEvent contains a viewer outbound message. */
export function isViewerMessage(event: MessageEvent): event is MessageEvent<ViewerOutboundMessage> {
	const data = event.data;
	return (
		typeof data === 'object' &&
		data !== null &&
		'type' in data &&
		(data.type === 'ready' ||
			data.type === 'error' ||
			data.type === 'zoneTapped' ||
			data.type === 'lightAngleChanged' ||
			data.type === 'tierChanged')
	);
}
