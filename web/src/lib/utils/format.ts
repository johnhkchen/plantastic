const UNIT_LABELS: Record<string, string> = {
	sq_ft: 'sq ft',
	cu_yd: 'cu yd',
	linear_ft: 'lin ft',
	each: 'each'
};

export function formatUnit(unit: string): string {
	return UNIT_LABELS[unit] ?? unit;
}

export function formatPrice(price: string): string {
	const n = parseFloat(price);
	return isNaN(n) ? price : `$${n.toFixed(2)}`;
}
