import { apiFetch } from '$lib/api';

export interface AssignmentResponse {
	zone_id: string;
	material_id: string;
	overrides: { price_override?: string; depth_override_inches?: number } | null;
}

export interface TierResponse {
	tier: 'good' | 'better' | 'best';
	assignments: AssignmentResponse[];
}

export interface AssignmentInput {
	zone_id: string;
	material_id: string;
	overrides?: { price_override?: string; depth_override_inches?: number } | null;
}

/** Fetch all three tiers with their assignments for a project. */
export function fetchTiers(projectId: string): Promise<TierResponse[]> {
	return apiFetch<TierResponse[]>(`/projects/${projectId}/tiers`);
}

/** Replace all assignments for a single tier. */
export async function saveTierAssignments(
	projectId: string,
	tier: string,
	assignments: AssignmentInput[]
): Promise<void> {
	await apiFetch<void>(`/projects/${projectId}/tiers/${tier}`, {
		method: 'PUT',
		body: { assignments }
	});
}
