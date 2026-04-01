/** Zone type enum matching backend ZoneType (snake_case serialization). */
export type ZoneType = 'bed' | 'patio' | 'path' | 'lawn' | 'wall' | 'edging';

/** All zone types in display order. */
export const ZONE_TYPES: ZoneType[] = ['bed', 'patio', 'path', 'lawn', 'wall', 'edging'];

/** Human-readable labels for zone types. */
export const ZONE_TYPE_LABELS: Record<ZoneType, string> = {
	bed: 'Bed',
	patio: 'Patio',
	path: 'Path',
	lawn: 'Lawn',
	wall: 'Wall',
	edging: 'Edging'
};

/** A point in canvas coordinate space. */
export interface Point {
	x: number;
	y: number;
}

/** A zone as stored during editing. */
export interface EditorZone {
	id: string;
	vertices: Point[];
	zoneType: ZoneType;
	label: string;
}

/** Component interaction modes. */
export type EditorMode = 'idle' | 'drawing' | 'selected';

/** Tracks an in-progress vertex drag. */
export interface DragState {
	zoneId: string;
	vertexIndex: number;
}

/** Data bundle for the renderer. */
export interface RenderState {
	zones: EditorZone[];
	selectedZoneId: string | null;
	drawingVertices: Point[];
	mousePos: Point | null;
	activeZoneType: ZoneType;
	mode: EditorMode;
}
