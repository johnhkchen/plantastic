import { apiFetch } from '$lib/api';

export interface LineItem {
	zone_id: string;
	zone_label: string | null;
	material_id: string;
	material_name: string;
	quantity: string;
	unit: string;
	unit_price: string;
	line_total: string;
}

export interface Quote {
	tier: string;
	line_items: LineItem[];
	subtotal: string;
	tax: string | null;
	total: string;
}

/** Fetch computed quote for a single tier. */
export function fetchQuote(projectId: string, tier: string): Promise<Quote> {
	return apiFetch<Quote>(`/projects/${projectId}/quote/${tier}`);
}
