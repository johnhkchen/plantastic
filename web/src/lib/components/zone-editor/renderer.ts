import type { Point, EditorZone, RenderState } from './types';
import { getZoneColors } from './colors';

const GRID_SPACING = 20;
const GRID_COLOR = '#e5e7eb';
const HANDLE_RADIUS = 5;
const HANDLE_FILL = '#ffffff';
const HANDLE_STROKE = '#374151';
const SELECTED_STROKE_WIDTH = 2.5;
const NORMAL_STROKE_WIDTH = 1.5;
const CLOSE_THRESHOLD_VISUAL = 12;

/** Draw a light grid background. */
function drawGrid(ctx: CanvasRenderingContext2D, width: number, height: number): void {
	ctx.strokeStyle = GRID_COLOR;
	ctx.lineWidth = 0.5;

	for (let x = 0; x <= width; x += GRID_SPACING) {
		ctx.beginPath();
		ctx.moveTo(x, 0);
		ctx.lineTo(x, height);
		ctx.stroke();
	}
	for (let y = 0; y <= height; y += GRID_SPACING) {
		ctx.beginPath();
		ctx.moveTo(0, y);
		ctx.lineTo(width, y);
		ctx.stroke();
	}
}

/** Trace a polygon path from vertices (does not stroke or fill). */
function tracePath(ctx: CanvasRenderingContext2D, vertices: Point[]): void {
	if (vertices.length < 2) return;
	ctx.beginPath();
	ctx.moveTo(vertices[0].x, vertices[0].y);
	for (let i = 1; i < vertices.length; i++) {
		ctx.lineTo(vertices[i].x, vertices[i].y);
	}
	ctx.closePath();
}

/** Draw a completed zone polygon. */
function drawZone(ctx: CanvasRenderingContext2D, zone: EditorZone, isSelected: boolean): void {
	if (zone.vertices.length < 3) return;

	const colors = getZoneColors(zone.zoneType);

	tracePath(ctx, zone.vertices);
	ctx.fillStyle = colors.fill;
	ctx.fill();
	ctx.strokeStyle = colors.stroke;
	ctx.lineWidth = isSelected ? SELECTED_STROKE_WIDTH : NORMAL_STROKE_WIDTH;
	ctx.stroke();

	// Draw label in center if present
	if (zone.label) {
		const center = polygonCenter(zone.vertices);
		ctx.fillStyle = colors.stroke;
		ctx.font = '12px Inter, system-ui, sans-serif';
		ctx.textAlign = 'center';
		ctx.textBaseline = 'middle';
		ctx.fillText(zone.label, center.x, center.y);
	}
}

/** Draw vertex handles for a selected zone. */
function drawHandles(ctx: CanvasRenderingContext2D, vertices: Point[]): void {
	for (const v of vertices) {
		ctx.beginPath();
		ctx.arc(v.x, v.y, HANDLE_RADIUS, 0, Math.PI * 2);
		ctx.fillStyle = HANDLE_FILL;
		ctx.fill();
		ctx.strokeStyle = HANDLE_STROKE;
		ctx.lineWidth = 1.5;
		ctx.stroke();
	}
}

/** Draw the in-progress polygon being drawn. */
function drawDrawingPolygon(
	ctx: CanvasRenderingContext2D,
	vertices: Point[],
	mousePos: Point | null,
	zoneType: string
): void {
	if (vertices.length === 0) return;

	const colors = getZoneColors(zoneType as import('./types').ZoneType);

	// Draw placed edges
	ctx.beginPath();
	ctx.moveTo(vertices[0].x, vertices[0].y);
	for (let i = 1; i < vertices.length; i++) {
		ctx.lineTo(vertices[i].x, vertices[i].y);
	}

	// Preview edge to mouse
	if (mousePos) {
		ctx.lineTo(mousePos.x, mousePos.y);
	}

	ctx.strokeStyle = colors.stroke;
	ctx.lineWidth = 1.5;
	ctx.setLineDash([6, 4]);
	ctx.stroke();
	ctx.setLineDash([]);

	// Light fill if we have enough vertices
	if (vertices.length >= 3) {
		tracePath(ctx, vertices);
		ctx.fillStyle = colors.fill;
		ctx.fill();
	}

	// Draw vertex dots
	for (const v of vertices) {
		ctx.beginPath();
		ctx.arc(v.x, v.y, 4, 0, Math.PI * 2);
		ctx.fillStyle = colors.stroke;
		ctx.fill();
	}

	// Highlight first vertex when close enough to close
	if (mousePos && vertices.length >= 3) {
		const dx = mousePos.x - vertices[0].x;
		const dy = mousePos.y - vertices[0].y;
		const dist = Math.sqrt(dx * dx + dy * dy);
		if (dist <= CLOSE_THRESHOLD_VISUAL) {
			ctx.beginPath();
			ctx.arc(vertices[0].x, vertices[0].y, 8, 0, Math.PI * 2);
			ctx.strokeStyle = colors.stroke;
			ctx.lineWidth = 2;
			ctx.stroke();
		}
	}
}

/** Compute the centroid of a polygon (average of vertices). */
function polygonCenter(vertices: Point[]): Point {
	let sx = 0;
	let sy = 0;
	for (const v of vertices) {
		sx += v.x;
		sy += v.y;
	}
	return { x: sx / vertices.length, y: sy / vertices.length };
}

/** Full canvas redraw. */
export function redraw(ctx: CanvasRenderingContext2D, state: RenderState): void {
	const { width, height } = ctx.canvas;

	// Account for device pixel ratio scaling
	const dpr = window.devicePixelRatio || 1;
	const w = width / dpr;
	const h = height / dpr;

	ctx.clearRect(0, 0, width, height);

	ctx.save();
	ctx.scale(dpr, dpr);

	// Background grid
	drawGrid(ctx, w, h);

	// Completed zones
	for (const zone of state.zones) {
		const isSelected = zone.id === state.selectedZoneId;
		drawZone(ctx, zone, isSelected);
	}

	// Handles for selected zone
	if (state.selectedZoneId) {
		const sel = state.zones.find((z) => z.id === state.selectedZoneId);
		if (sel) {
			drawHandles(ctx, sel.vertices);
		}
	}

	// Drawing in progress
	if (state.mode === 'drawing') {
		drawDrawingPolygon(ctx, state.drawingVertices, state.mousePos, state.activeZoneType);
	}

	ctx.restore();
}
