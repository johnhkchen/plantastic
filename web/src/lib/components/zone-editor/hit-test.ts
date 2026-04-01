import type { Point, EditorZone } from './types';

/**
 * Ray-casting point-in-polygon test.
 * Casts a horizontal ray rightward from the point and counts edge crossings.
 * Odd count = inside, even count = outside.
 */
export function isPointInPolygon(point: Point, vertices: Point[]): boolean {
	const n = vertices.length;
	if (n < 3) return false;

	let inside = false;
	for (let i = 0, j = n - 1; i < n; j = i++) {
		const vi = vertices[i];
		const vj = vertices[j];

		if (vi.y > point.y !== vj.y > point.y) {
			const intersectX = vj.x + ((point.y - vj.y) / (vi.y - vj.y)) * (vi.x - vj.x);
			if (point.x < intersectX) {
				inside = !inside;
			}
		}
	}
	return inside;
}

/** Distance between two points. */
export function distance(a: Point, b: Point): number {
	const dx = a.x - b.x;
	const dy = a.y - b.y;
	return Math.sqrt(dx * dx + dy * dy);
}

/** Find the nearest vertex to a point. Returns index and distance. */
export function nearestVertex(
	point: Point,
	vertices: Point[]
): { index: number; distance: number } {
	let minDist = Infinity;
	let minIndex = -1;

	for (let i = 0; i < vertices.length; i++) {
		const d = distance(point, vertices[i]);
		if (d < minDist) {
			minDist = d;
			minIndex = i;
		}
	}

	return { index: minIndex, distance: minDist };
}

/** Check if a point is close enough to the first vertex to close a polygon. */
export function isNearFirstVertex(point: Point, vertices: Point[], threshold: number): boolean {
	if (vertices.length < 3) return false;
	return distance(point, vertices[0]) <= threshold;
}

/**
 * Find which zone contains a point. Checks in reverse order so the
 * topmost (most recently created) zone is found first.
 */
export function findZoneAtPoint(point: Point, zones: EditorZone[]): EditorZone | null {
	for (let i = zones.length - 1; i >= 0; i--) {
		if (isPointInPolygon(point, zones[i].vertices)) {
			return zones[i];
		}
	}
	return null;
}
