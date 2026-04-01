import type { EditorZone } from '$lib/components/zone-editor/types';
import { apiFetch } from '$lib/api';
import type { ApiZone } from './types';
import { editorZoneToGeoJson } from './types';

/** Fetch all zones for a project. */
export async function fetchZones(projectId: string): Promise<ApiZone[]> {
	return apiFetch<ApiZone[]>(`/projects/${projectId}/zones`);
}

/** Bulk-save zones for a project (replaces all), then re-fetch to get computed fields. */
export async function saveZones(projectId: string, zones: EditorZone[]): Promise<ApiZone[]> {
	const body = zones.map((z, i) => ({
		geometry: editorZoneToGeoJson(z),
		zone_type: z.zoneType,
		label: z.label || null,
		sort_order: i
	}));

	await apiFetch<string[]>(`/projects/${projectId}/zones`, {
		method: 'PUT',
		body
	});

	// Re-fetch to get server-generated IDs and computed measurements
	return fetchZones(projectId);
}

/** Delete a single zone. */
export async function deleteZone(projectId: string, zoneId: string): Promise<void> {
	await apiFetch<void>(`/projects/${projectId}/zones/${zoneId}`, { method: 'DELETE' });
}
