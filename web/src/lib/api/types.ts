import type { EditorZone, Point } from '$lib/components/zone-editor/types';

/** GeoJSON Polygon geometry. */
export interface GeoJsonPolygon {
	type: 'Polygon';
	coordinates: number[][][];
}

/** Zone as returned by the API. */
export interface ApiZone {
	id: string;
	project_id: string;
	geometry: GeoJsonPolygon;
	zone_type: 'bed' | 'patio' | 'path' | 'lawn' | 'wall' | 'edging';
	label: string | null;
	sort_order: number;
	area_sqft: number;
	perimeter_ft: number;
	created_at: string;
	updated_at: string;
}

/** Convert an EditorZone (canvas coords) to a GeoJSON Polygon. */
export function editorZoneToGeoJson(zone: EditorZone): GeoJsonPolygon {
	const ring = zone.vertices.map((v) => [v.x, v.y]);
	// Close the ring
	if (ring.length > 0) {
		ring.push([ring[0][0], ring[0][1]]);
	}
	return { type: 'Polygon', coordinates: [ring] };
}

/** Convert an ApiZone (GeoJSON) back to an EditorZone (canvas coords). */
export function apiZoneToEditorZone(zone: ApiZone): EditorZone {
	const coords = zone.geometry.coordinates[0] ?? [];
	// Remove the closing coordinate (last == first in GeoJSON rings)
	const vertices: Point[] =
		coords.length > 1
			? coords.slice(0, -1).map(([x, y]) => ({ x, y }))
			: coords.map(([x, y]) => ({ x, y }));

	return {
		id: zone.id,
		vertices,
		zoneType: zone.zone_type,
		label: zone.label ?? ''
	};
}
