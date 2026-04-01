<script lang="ts">
	import {
		ZONE_TYPES,
		ZONE_TYPE_LABELS,
		type EditorZone,
		type ZoneType,
		type EditorMode,
		type Point,
		type DragState
	} from './types';
	import { getZoneColors } from './colors';
	import { findZoneAtPoint, isNearFirstVertex, nearestVertex } from './hit-test';
	import { redraw } from './renderer';

	const VERTEX_HIT_RADIUS = 10;
	const CLOSE_HIT_RADIUS = 12;

	let { zones = $bindable<EditorZone[]>([]) }: { zones: EditorZone[] } = $props();

	let mode = $state<EditorMode>('idle');
	let activeZoneType = $state<ZoneType>('bed');
	let drawingVertices = $state<Point[]>([]);
	let selectedZoneId = $state<string | null>(null);
	let dragState = $state<DragState | null>(null);
	let mousePos = $state<Point | null>(null);
	let canvasEl: HTMLCanvasElement | undefined = $state();

	let selectedZone = $derived(zones.find((z) => z.id === selectedZoneId) ?? null);

	// Reactive canvas redraw
	$effect(() => {
		if (!canvasEl) return;
		const ctx = canvasEl.getContext('2d');
		if (!ctx) return;

		// Touch all reactive state to establish dependencies
		const _trigger = [
			zones.map((z) => [z.vertices, z.zoneType, z.label, z.id]),
			selectedZoneId,
			drawingVertices,
			mousePos,
			activeZoneType,
			mode
		];
		void _trigger;

		redraw(ctx, {
			zones,
			selectedZoneId,
			drawingVertices,
			mousePos,
			activeZoneType,
			mode
		});
	});

	// Canvas sizing with DPI scaling
	$effect(() => {
		if (!canvasEl) return;
		const parent = canvasEl.parentElement;
		if (!parent) return;

		const observer = new ResizeObserver(() => sizeCanvas());
		observer.observe(parent);
		sizeCanvas();

		return () => observer.disconnect();
	});

	function sizeCanvas(): void {
		if (!canvasEl) return;
		const parent = canvasEl.parentElement;
		if (!parent) return;

		const dpr = window.devicePixelRatio || 1;
		const rect = parent.getBoundingClientRect();
		canvasEl.width = rect.width * dpr;
		canvasEl.height = rect.height * dpr;
		canvasEl.style.width = `${rect.width}px`;
		canvasEl.style.height = `${rect.height}px`;
	}

	function getCanvasPoint(e: MouseEvent): Point {
		if (!canvasEl) return { x: 0, y: 0 };
		const rect = canvasEl.getBoundingClientRect();
		return {
			x: e.clientX - rect.left,
			y: e.clientY - rect.top
		};
	}

	function generateId(): string {
		return crypto.randomUUID();
	}

	// --- Drawing ---

	function finishDrawing(): void {
		if (drawingVertices.length < 3) {
			cancelDrawing();
			return;
		}
		const newZone: EditorZone = {
			id: generateId(),
			vertices: [...drawingVertices],
			zoneType: activeZoneType,
			label: ''
		};
		zones = [...zones, newZone];
		drawingVertices = [];
		selectedZoneId = newZone.id;
		mode = 'selected';
	}

	function cancelDrawing(): void {
		drawingVertices = [];
		mode = 'idle';
	}

	// --- Zone operations ---

	function deleteSelectedZone(): void {
		if (!selectedZoneId) return;
		zones = zones.filter((z) => z.id !== selectedZoneId);
		selectedZoneId = null;
		mode = 'idle';
	}

	function updateSelectedLabel(label: string): void {
		if (!selectedZoneId) return;
		zones = zones.map((z) => (z.id === selectedZoneId ? { ...z, label } : z));
	}

	function updateSelectedType(zoneType: ZoneType): void {
		if (!selectedZoneId) return;
		zones = zones.map((z) => (z.id === selectedZoneId ? { ...z, zoneType } : z));
	}

	// --- Event handlers ---

	function handleClick(e: MouseEvent): void {
		// Ignore if this was the end of a drag
		if (dragState) return;

		const pt = getCanvasPoint(e);

		if (mode === 'drawing') {
			// Close polygon if clicking near first vertex
			if (isNearFirstVertex(pt, drawingVertices, CLOSE_HIT_RADIUS)) {
				finishDrawing();
				return;
			}
			drawingVertices = [...drawingVertices, pt];
			return;
		}

		// Check if clicking on an existing zone
		const hitZone = findZoneAtPoint(pt, zones);
		if (hitZone) {
			selectedZoneId = hitZone.id;
			mode = 'selected';
			return;
		}

		// Click on empty space
		if (mode === 'selected') {
			selectedZoneId = null;
			mode = 'idle';
			return;
		}

		// Start drawing a new polygon
		drawingVertices = [pt];
		mode = 'drawing';
	}

	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	function handleDblClick(_e: MouseEvent): void {
		if (mode !== 'drawing') return;

		// Double-click fires two clicks first, so the last two vertices are
		// the double-click position — remove the duplicate before closing.
		if (drawingVertices.length > 1) {
			drawingVertices = drawingVertices.slice(0, -1);
		}
		finishDrawing();
	}

	function handleMouseMove(e: MouseEvent): void {
		const pt = getCanvasPoint(e);
		mousePos = pt;

		if (dragState && selectedZoneId) {
			zones = zones.map((z) => {
				if (z.id !== dragState!.zoneId) return z;
				const newVerts = [...z.vertices];
				newVerts[dragState!.vertexIndex] = pt;
				return { ...z, vertices: newVerts };
			});
		}
	}

	function handleMouseDown(e: MouseEvent): void {
		if (mode !== 'selected' || !selectedZoneId) return;

		const pt = getCanvasPoint(e);
		const sel = zones.find((z) => z.id === selectedZoneId);
		if (!sel) return;

		// Check if mousedown is on a vertex handle
		const nearest = nearestVertex(pt, sel.vertices);
		if (nearest.distance <= VERTEX_HIT_RADIUS) {
			dragState = { zoneId: selectedZoneId, vertexIndex: nearest.index };
			e.preventDefault();
		}
	}

	function handleMouseUp(): void {
		if (dragState) {
			// Small delay so the click handler can see dragState was active
			const ds = dragState;
			dragState = null;
			// Prevent the click handler from firing after drag
			setTimeout(() => void ds, 0);
		}
	}

	function handleKeyDown(e: KeyboardEvent): void {
		if (e.key === 'Escape') {
			if (mode === 'drawing') {
				cancelDrawing();
			} else if (mode === 'selected') {
				selectedZoneId = null;
				mode = 'idle';
			}
		}
		if ((e.key === 'Delete' || e.key === 'Backspace') && mode === 'selected') {
			// Don't delete if user is typing in the label input
			if ((e.target as HTMLElement)?.tagName === 'INPUT') return;
			deleteSelectedZone();
		}
	}

	// Status text
	let statusText = $derived.by(() => {
		switch (mode) {
			case 'idle':
				return 'Click to start drawing a zone';
			case 'drawing': {
				const n = drawingVertices.length;
				if (n < 3) return `Click to place vertices (${n}/3 minimum)`;
				return 'Click to add vertices \u2022 Click first vertex or double-click to close';
			}
			case 'selected':
				return 'Drag handles to reshape \u2022 Delete to remove \u2022 Escape to deselect';
		}
	});
</script>

<svelte:window onkeydown={handleKeyDown} />

<div class="flex h-full flex-col">
	<!-- Toolbar -->
	<div class="flex items-center gap-3 border-b border-gray-200 bg-white px-4 py-2">
		<!-- Zone type selector -->
		<span class="text-xs font-medium text-gray-500 uppercase">Zone type:</span>
		<div class="flex gap-1">
			{#each ZONE_TYPES as zt (zt)}
				{@const colors = getZoneColors(zt)}
				{@const isActive =
					mode === 'selected' && selectedZone
						? selectedZone.zoneType === zt
						: activeZoneType === zt}
				<button
					type="button"
					class="rounded px-2.5 py-1 text-xs font-medium transition-colors"
					style:background-color={isActive ? colors.stroke : 'transparent'}
					style:color={isActive ? 'white' : colors.stroke}
					style:border={`1.5px solid ${colors.stroke}`}
					onclick={() => {
						if (mode === 'selected' && selectedZoneId) {
							updateSelectedType(zt);
						} else {
							activeZoneType = zt;
						}
					}}
				>
					{ZONE_TYPE_LABELS[zt]}
				</button>
			{/each}
		</div>

		<!-- Separator -->
		<div class="mx-1 h-6 w-px bg-gray-200"></div>

		<!-- Label input (when zone selected) -->
		{#if mode === 'selected' && selectedZone}
			<label class="flex items-center gap-1.5">
				<span class="text-xs font-medium text-gray-500">Label:</span>
				<input
					type="text"
					class="rounded border border-gray-300 px-2 py-1 text-xs focus:border-brand-primary focus:outline-none"
					placeholder="Optional label"
					value={selectedZone.label}
					oninput={(e) => updateSelectedLabel((e.target as HTMLInputElement).value)}
				/>
			</label>

			<button
				type="button"
				class="ml-auto rounded bg-red-50 px-2.5 py-1 text-xs font-medium text-red-600 transition-colors hover:bg-red-100"
				onclick={deleteSelectedZone}
			>
				Delete zone
			</button>
		{/if}
	</div>

	<!-- Canvas container -->
	<div class="relative min-h-0 flex-1 bg-gray-50">
		<canvas
			bind:this={canvasEl}
			onclick={handleClick}
			ondblclick={handleDblClick}
			onmousemove={handleMouseMove}
			onmousedown={handleMouseDown}
			onmouseup={handleMouseUp}
			oncontextmenu={(e) => e.preventDefault()}
			class="absolute inset-0 cursor-crosshair"
			style:cursor={mode === 'selected' && dragState
				? 'grabbing'
				: mode === 'selected'
					? 'default'
					: 'crosshair'}
		></canvas>
	</div>

	<!-- Status bar -->
	<div class="flex items-center justify-between border-t border-gray-200 bg-white px-4 py-1.5">
		<span class="text-xs text-gray-500">{statusText}</span>
		<span class="text-xs text-gray-400">{zones.length} zone{zones.length !== 1 ? 's' : ''}</span>
	</div>
</div>
