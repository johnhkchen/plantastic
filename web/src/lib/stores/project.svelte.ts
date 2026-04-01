export interface DetectedTree {
	location: { latitude: number; longitude: number };
	height_ft: number;
	spread_ft: number;
	confidence: number;
}

export interface ProjectBaseline {
	coordinates: { latitude: number; longitude: number };
	lot_boundary: {
		polygon: unknown;
		area_sqft: number;
		source: string;
	};
	trees: DetectedTree[];
	sun_grid: {
		bounds: { south: number; west: number; north: number; east: number };
		resolution_meters: number;
		width: number;
		height: number;
		values: number[];
		sample_days_used: number;
	};
}

export interface Project {
	id: string;
	tenant_id: string;
	client_name: string | null;
	client_email: string | null;
	address: string | null;
	baseline: ProjectBaseline | null;
	status: 'draft' | 'quoted' | 'approved' | 'complete';
	created_at: string;
	updated_at: string;
}

export interface Material {
	id: string;
	tenant_id: string;
	name: string;
	category: 'hardscape' | 'softscape' | 'edging' | 'fill';
	unit: 'sq_ft' | 'cu_yd' | 'linear_ft' | 'each';
	price_per_unit: string;
	depth_inches: number | null;
	extrusion: unknown;
	texture_key: string | null;
	photo_key: string | null;
	supplier_sku: string | null;
	created_at: string;
	updated_at: string;
}

export interface Zone {
	id: string;
	vertices: { x: number; y: number }[];
	zoneType: 'bed' | 'patio' | 'path' | 'lawn' | 'wall' | 'edging';
	label: string;
}

export interface Tier {
	id: string;
	name: string;
	plantIds: string[];
}

let projects = $state<Project[]>([]);
let current = $state<Project | null>(null);
let zones = $state<Zone[]>([]);
let tiers = $state<Tier[]>([]);

export const projectStore = {
	get projects() {
		return projects;
	},
	set projects(v: Project[]) {
		projects = v;
	},
	get current() {
		return current;
	},
	set current(v: Project | null) {
		current = v;
	},
	get zones() {
		return zones;
	},
	set zones(v: Zone[]) {
		zones = v;
	},
	get tiers() {
		return tiers;
	},
	set tiers(v: Tier[]) {
		tiers = v;
	},
	reset() {
		current = null;
		zones = [];
		tiers = [];
	}
};
