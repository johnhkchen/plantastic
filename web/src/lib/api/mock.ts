import type { ApiOptions, SseOptions } from './client';
import type { Project, Material } from '$lib/stores/project.svelte';

const TENANT_ID = '00000000-0000-0000-0000-000000000001';

const MOCK_PROJECTS: Project[] = [
	{
		id: 'a1111111-1111-1111-1111-111111111111',
		tenant_id: TENANT_ID,
		client_name: 'Maria Santos',
		client_email: 'maria@example.com',
		address: '742 Evergreen Terrace, San Jose, CA',
		baseline: null,
		status: 'draft',
		created_at: '2026-03-15T10:00:00Z',
		updated_at: '2026-03-15T10:00:00Z'
	},
	{
		id: 'a2222222-2222-2222-2222-222222222222',
		tenant_id: TENANT_ID,
		client_name: 'James Chen',
		client_email: 'james@example.com',
		address: '123 Oak Ave, Palo Alto, CA',
		baseline: null,
		status: 'quoted',
		created_at: '2026-03-20T14:30:00Z',
		updated_at: '2026-03-22T09:00:00Z'
	},
	{
		id: 'a3333333-3333-3333-3333-333333333333',
		tenant_id: TENANT_ID,
		client_name: 'Priya Patel',
		client_email: null,
		address: '456 Willow Dr, Mountain View, CA',
		baseline: null,
		status: 'approved',
		created_at: '2026-03-25T09:15:00Z',
		updated_at: '2026-03-28T11:00:00Z'
	}
];

const MOCK_MATERIALS: Material[] = [
	{
		id: 'b1111111-1111-1111-1111-111111111111',
		tenant_id: TENANT_ID,
		name: 'Flagstone Pavers',
		category: 'hardscape',
		unit: 'sq_ft',
		price_per_unit: '8.50',
		depth_inches: 2.0,
		extrusion: { type: 'SitsOnTop', height_inches: 2.0 },
		texture_key: null,
		photo_key: null,
		supplier_sku: 'FLG-001',
		created_at: '2026-03-10T08:00:00Z',
		updated_at: '2026-03-10T08:00:00Z'
	},
	{
		id: 'b2222222-2222-2222-2222-222222222222',
		tenant_id: TENANT_ID,
		name: 'Decomposed Granite',
		category: 'fill',
		unit: 'cu_yd',
		price_per_unit: '45.00',
		depth_inches: 3.0,
		extrusion: { type: 'Fills', flush: true },
		texture_key: null,
		photo_key: null,
		supplier_sku: 'DG-003',
		created_at: '2026-03-10T08:00:00Z',
		updated_at: '2026-03-10T08:00:00Z'
	},
	{
		id: 'b3333333-3333-3333-3333-333333333333',
		tenant_id: TENANT_ID,
		name: 'Japanese Boxwood',
		category: 'softscape',
		unit: 'each',
		price_per_unit: '24.00',
		depth_inches: null,
		extrusion: { type: 'SitsOnTop', height_inches: 18.0 },
		texture_key: null,
		photo_key: null,
		supplier_sku: null,
		created_at: '2026-03-12T10:00:00Z',
		updated_at: '2026-03-12T10:00:00Z'
	},
	{
		id: 'b4444444-4444-4444-4444-444444444444',
		tenant_id: TENANT_ID,
		name: 'Steel Edging',
		category: 'edging',
		unit: 'linear_ft',
		price_per_unit: '3.75',
		depth_inches: 4.0,
		extrusion: { type: 'BuildsUp', height_inches: 4.0 },
		texture_key: null,
		photo_key: null,
		supplier_sku: 'SE-010',
		created_at: '2026-03-14T12:00:00Z',
		updated_at: '2026-03-14T12:00:00Z'
	},
	{
		id: 'b5555555-5555-5555-5555-555555555555',
		tenant_id: TENANT_ID,
		name: 'River Rock',
		category: 'fill',
		unit: 'cu_yd',
		price_per_unit: '55.00',
		depth_inches: 3.0,
		extrusion: { type: 'Fills', flush: false },
		texture_key: null,
		photo_key: null,
		supplier_sku: 'RR-200',
		created_at: '2026-03-16T08:00:00Z',
		updated_at: '2026-03-16T08:00:00Z'
	},
	{
		id: 'b6666666-6666-6666-6666-666666666666',
		tenant_id: TENANT_ID,
		name: 'Lavender',
		category: 'softscape',
		unit: 'each',
		price_per_unit: '12.00',
		depth_inches: null,
		extrusion: { type: 'SitsOnTop', height_inches: 24.0 },
		texture_key: null,
		photo_key: null,
		supplier_sku: null,
		created_at: '2026-03-18T09:00:00Z',
		updated_at: '2026-03-18T09:00:00Z'
	},
	{
		id: 'b7777777-7777-7777-7777-777777777777',
		tenant_id: TENANT_ID,
		name: 'Concrete Pavers',
		category: 'hardscape',
		unit: 'sq_ft',
		price_per_unit: '6.25',
		depth_inches: 2.5,
		extrusion: { type: 'SitsOnTop', height_inches: 2.5 },
		texture_key: null,
		photo_key: null,
		supplier_sku: 'CP-100',
		created_at: '2026-03-20T10:00:00Z',
		updated_at: '2026-03-20T10:00:00Z'
	}
];

/** Shoelace formula for polygon area from GeoJSON coordinates ring. */
function mockArea(ring: number[][]): number {
	let area = 0;
	const n = ring.length;
	for (let i = 0; i < n; i++) {
		const j = (i + 1) % n;
		area += ring[i][0] * ring[j][1];
		area -= ring[j][0] * ring[i][1];
	}
	return Math.abs(area) / 2;
}

/** Sum of edge lengths for polygon perimeter from GeoJSON coordinates ring. */
function mockPerimeter(ring: number[][]): number {
	let perim = 0;
	for (let i = 0; i < ring.length - 1; i++) {
		const dx = ring[i + 1][0] - ring[i][0];
		const dy = ring[i + 1][1] - ring[i][1];
		perim += Math.sqrt(dx * dx + dy * dy);
	}
	return perim;
}

interface MockApiZone {
	id: string;
	project_id: string;
	geometry: { type: 'Polygon'; coordinates: number[][][] };
	zone_type: string;
	label: string | null;
	sort_order: number;
	area_sqft: number;
	perimeter_ft: number;
	created_at: string;
	updated_at: string;
}

let mockZones: MockApiZone[] = [
	{
		id: 'zone-1',
		project_id: 'a1111111-1111-1111-1111-111111111111',
		geometry: {
			type: 'Polygon',
			coordinates: [
				[
					[100, 100],
					[220, 100],
					[220, 200],
					[100, 200],
					[100, 100]
				]
			]
		},
		zone_type: 'bed',
		label: 'Front Border',
		sort_order: 0,
		area_sqft: 12000,
		perimeter_ft: 440,
		created_at: '2026-03-15T10:00:00Z',
		updated_at: '2026-03-15T10:00:00Z'
	},
	{
		id: 'zone-2',
		project_id: 'a1111111-1111-1111-1111-111111111111',
		geometry: {
			type: 'Polygon',
			coordinates: [
				[
					[300, 150],
					[500, 150],
					[500, 350],
					[300, 350],
					[300, 150]
				]
			]
		},
		zone_type: 'patio',
		label: 'Back Patio',
		sort_order: 1,
		area_sqft: 40000,
		perimeter_ft: 800,
		created_at: '2026-03-15T10:00:00Z',
		updated_at: '2026-03-15T10:00:00Z'
	}
];

// Tier assignments: keyed by "projectId:tier"
const mockTierAssignments: Record<
	string,
	{ zone_id: string; material_id: string; overrides: unknown | null }[]
> = {};

let nextProjectNum = 4;
let nextMaterialNum = 8;

function delay(ms = 150): Promise<void> {
	return new Promise((r) => setTimeout(r, ms + Math.random() * 150));
}

export async function mockApiFetch<T>(path: string, options: ApiOptions = {}): Promise<T> {
	await delay();
	const method = options.method ?? 'GET';

	// POST /projects
	if (path === '/projects' && method === 'POST') {
		const body = options.body as Record<string, unknown> | undefined;
		const project: Project = {
			id: `a${nextProjectNum}${nextProjectNum}${nextProjectNum}${nextProjectNum}${nextProjectNum}${nextProjectNum}${nextProjectNum}${nextProjectNum}-mock`,
			tenant_id: TENANT_ID,
			client_name: (body?.client_name as string) ?? null,
			client_email: (body?.client_email as string) ?? null,
			address: (body?.address as string) ?? null,
			baseline: null,
			status: 'draft',
			created_at: new Date().toISOString(),
			updated_at: new Date().toISOString()
		};
		nextProjectNum++;
		MOCK_PROJECTS.push(project);
		return project as T;
	}

	// GET /projects
	if (path === '/projects' && method === 'GET') {
		return MOCK_PROJECTS as T;
	}

	// GET /projects/:id
	const projectMatch = path.match(/^\/projects\/([\w-]+)$/);
	if (projectMatch && method === 'GET') {
		const project = MOCK_PROJECTS.find((p) => p.id === projectMatch[1]);
		if (project) return project as T;
		throw new Error(`Not found: ${path}`);
	}

	// DELETE /projects/:id
	if (projectMatch && method === 'DELETE') {
		const idx = MOCK_PROJECTS.findIndex((p) => p.id === projectMatch[1]);
		if (idx >= 0) MOCK_PROJECTS.splice(idx, 1);
		return undefined as T;
	}

	// POST /materials
	if (path === '/materials' && method === 'POST') {
		const body = options.body as Record<string, unknown> | undefined;
		const material: Material = {
			id: `b${nextMaterialNum}${nextMaterialNum}${nextMaterialNum}${nextMaterialNum}${nextMaterialNum}${nextMaterialNum}${nextMaterialNum}${nextMaterialNum}-mock`,
			tenant_id: TENANT_ID,
			name: (body?.name as string) ?? '',
			category: (body?.category as Material['category']) ?? 'hardscape',
			unit: (body?.unit as Material['unit']) ?? 'sq_ft',
			price_per_unit: String(body?.price_per_unit ?? '0'),
			depth_inches: (body?.depth_inches as number) ?? null,
			extrusion: body?.extrusion ?? { type: 'Fills', flush: true },
			texture_key: null,
			photo_key: null,
			supplier_sku: (body?.supplier_sku as string) ?? null,
			created_at: new Date().toISOString(),
			updated_at: new Date().toISOString()
		};
		nextMaterialNum++;
		MOCK_MATERIALS.push(material);
		return material as T;
	}

	// GET /materials
	if (path === '/materials' && method === 'GET') {
		return MOCK_MATERIALS as T;
	}

	// PATCH /materials/:id
	const materialMatch = path.match(/^\/materials\/([\w-]+)$/);
	if (materialMatch && method === 'PATCH') {
		const mat = MOCK_MATERIALS.find((m) => m.id === materialMatch[1]);
		if (mat) {
			const body = options.body as Record<string, unknown> | undefined;
			if (body) {
				Object.assign(mat, body, { updated_at: new Date().toISOString() });
			}
		}
		return undefined as T;
	}

	// DELETE /materials/:id
	if (materialMatch && method === 'DELETE') {
		const idx = MOCK_MATERIALS.findIndex((m) => m.id === materialMatch[1]);
		if (idx >= 0) MOCK_MATERIALS.splice(idx, 1);
		return undefined as T;
	}

	// GET /projects/:id/zones
	const zoneListMatch = path.match(/^\/projects\/([\w-]+)\/zones$/);
	if (zoneListMatch && method === 'GET') {
		const projId = zoneListMatch[1];
		return mockZones.filter((z) => z.project_id === projId) as T;
	}

	// POST /projects/:id/zones (add single zone)
	if (zoneListMatch && method === 'POST') {
		const projId = zoneListMatch[1];
		const body = options.body as Record<string, unknown> | undefined;
		if (body) {
			const geom = body.geometry as { type: 'Polygon'; coordinates: number[][][] };
			const ring = geom.coordinates[0] ?? [];
			const zone: MockApiZone = {
				id: crypto.randomUUID(),
				project_id: projId,
				geometry: geom,
				zone_type: (body.zone_type as string) ?? 'bed',
				label: (body.label as string) ?? null,
				sort_order: (body.sort_order as number) ?? 0,
				area_sqft: mockArea(ring),
				perimeter_ft: mockPerimeter(ring),
				created_at: new Date().toISOString(),
				updated_at: new Date().toISOString()
			};
			mockZones.push(zone);
			return zone as T;
		}
	}

	// PUT /projects/:id/zones (bulk replace)
	if (zoneListMatch && method === 'PUT') {
		const projId = zoneListMatch[1];
		const body = options.body as
			| {
					geometry: { type: 'Polygon'; coordinates: number[][][] };
					zone_type: string;
					label: string | null;
					sort_order: number;
			  }[]
			| undefined;
		// Remove existing zones for this project
		mockZones = mockZones.filter((z) => z.project_id !== projId);
		const newIds: string[] = [];
		if (body) {
			for (const entry of body) {
				const ring = entry.geometry.coordinates[0] ?? [];
				const zone: MockApiZone = {
					id: crypto.randomUUID(),
					project_id: projId,
					geometry: entry.geometry,
					zone_type: entry.zone_type,
					label: entry.label,
					sort_order: entry.sort_order ?? 0,
					area_sqft: mockArea(ring),
					perimeter_ft: mockPerimeter(ring),
					created_at: new Date().toISOString(),
					updated_at: new Date().toISOString()
				};
				mockZones.push(zone);
				newIds.push(zone.id);
			}
		}
		return newIds as T;
	}

	// PATCH /projects/:id/zones/:zid
	const zoneSingleMatch = path.match(/^\/projects\/([\w-]+)\/zones\/([\w-]+)$/);
	if (zoneSingleMatch && method === 'PATCH') {
		const zoneId = zoneSingleMatch[2];
		const body = options.body as Record<string, unknown> | undefined;
		const zone = mockZones.find((z) => z.id === zoneId);
		if (zone && body) {
			if (body.geometry) {
				zone.geometry = body.geometry as MockApiZone['geometry'];
				const ring = zone.geometry.coordinates[0] ?? [];
				zone.area_sqft = mockArea(ring);
				zone.perimeter_ft = mockPerimeter(ring);
			}
			if (body.zone_type) zone.zone_type = body.zone_type as string;
			if ('label' in body) zone.label = body.label as string | null;
			if (body.sort_order != null) zone.sort_order = body.sort_order as number;
			zone.updated_at = new Date().toISOString();
		}
		return undefined as T;
	}

	// DELETE /projects/:id/zones/:zid
	if (zoneSingleMatch && method === 'DELETE') {
		const zoneId = zoneSingleMatch[2];
		mockZones = mockZones.filter((z) => z.id !== zoneId);
		return undefined as T;
	}

	// GET /projects/:id/tiers
	const tiersListMatch = path.match(/^\/projects\/([\w-]+)\/tiers$/);
	if (tiersListMatch && method === 'GET') {
		const projId = tiersListMatch[1];
		const tiers = ['good', 'better', 'best'].map((tier) => ({
			tier,
			assignments: mockTierAssignments[`${projId}:${tier}`] ?? []
		}));
		return tiers as T;
	}

	// PUT /projects/:id/tiers/:tier
	const tierSetMatch = path.match(/^\/projects\/([\w-]+)\/tiers\/(good|better|best)$/);
	if (tierSetMatch && method === 'PUT') {
		const projId = tierSetMatch[1];
		const tier = tierSetMatch[2];
		const body = options.body as
			| { assignments: { zone_id: string; material_id: string; overrides?: unknown }[] }
			| undefined;
		mockTierAssignments[`${projId}:${tier}`] = (body?.assignments ?? []).map((a) => ({
			zone_id: a.zone_id,
			material_id: a.material_id,
			overrides: a.overrides ?? null
		}));
		return undefined as T;
	}

	// GET /projects/:id/quote/:tier
	const quoteMatch = path.match(/^\/projects\/([\w-]+)\/quote\/(good|better|best)$/);
	if (quoteMatch && method === 'GET') {
		const projId = quoteMatch[1];
		const tier = quoteMatch[2];
		const assignments = mockTierAssignments[`${projId}:${tier}`] ?? [];
		const projZones = mockZones.filter((z) => z.project_id === projId);

		const lineItems = assignments
			.map((a) => {
				const zone = projZones.find((z) => z.id === a.zone_id);
				const mat = MOCK_MATERIALS.find((m) => m.id === a.material_id);
				if (!zone || !mat) return null;

				let quantity: number;
				switch (mat.unit) {
					case 'sq_ft':
						quantity = zone.area_sqft;
						break;
					case 'linear_ft':
						quantity = zone.perimeter_ft;
						break;
					case 'each':
						quantity = 1;
						break;
					case 'cu_yd':
						quantity = (zone.area_sqft * ((mat.depth_inches ?? 0) / 12)) / 27;
						break;
					default:
						quantity = 0;
				}

				const price = parseFloat(mat.price_per_unit);
				const lineTotal = quantity * price;

				return {
					zone_id: a.zone_id,
					zone_label: zone.label,
					material_id: a.material_id,
					material_name: mat.name,
					quantity: quantity.toFixed(4),
					unit: mat.unit,
					unit_price: mat.price_per_unit,
					line_total: lineTotal.toFixed(2)
				};
			})
			.filter(Boolean);

		const subtotal = lineItems.reduce((sum, li) => sum + parseFloat(li!.line_total), 0);

		return {
			tier,
			line_items: lineItems,
			subtotal: subtotal.toFixed(2),
			tax: null,
			total: subtotal.toFixed(2)
		} as T;
	}

	throw new Error(`Mock: no handler for ${method} ${path}`);
}

export async function mockSseStream(path: string, options: SseOptions): Promise<void> {
	const { onEvent, onDone } = options;
	const events = [
		{ type: 'start', message: 'Processing...' },
		{ type: 'progress', message: 'Analyzing zones...' },
		{ type: 'progress', message: 'Calculating materials...' },
		{ type: 'done', message: 'Complete' }
	];

	for (const event of events) {
		await delay(200);
		onEvent(event);
	}

	onDone?.();
}
