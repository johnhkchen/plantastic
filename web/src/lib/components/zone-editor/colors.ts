import type { ZoneType } from './types';

export interface ZoneColors {
	fill: string;
	stroke: string;
}

/** Color mapping for each zone type. Fill colors use 30% alpha for translucency. */
export const ZONE_COLOR_MAP: Record<ZoneType, ZoneColors> = {
	bed: { fill: 'rgba(139, 69, 19, 0.3)', stroke: '#8B4513' },
	patio: { fill: 'rgba(128, 128, 128, 0.3)', stroke: '#808080' },
	path: { fill: 'rgba(210, 180, 140, 0.3)', stroke: '#D2B48C' },
	lawn: { fill: 'rgba(34, 139, 34, 0.3)', stroke: '#228B22' },
	wall: { fill: 'rgba(105, 105, 105, 0.3)', stroke: '#696969' },
	edging: { fill: 'rgba(160, 82, 45, 0.3)', stroke: '#A0522D' }
};

export function getZoneColors(type: ZoneType): ZoneColors {
	return ZONE_COLOR_MAP[type];
}
