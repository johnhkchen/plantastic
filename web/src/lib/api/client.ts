import { session } from '$lib/stores/session.svelte';
import { ApiError, AuthError, RateLimitError, ValidationError } from './errors';

export interface ApiOptions {
	method?: string;
	body?: unknown;
	headers?: Record<string, string>;
}

export interface SseOptions {
	method?: string;
	body?: unknown;
	onEvent: (data: unknown) => void;
	onError?: (error: Error) => void;
	onDone?: () => void;
}

function baseUrl(): string {
	return '/api';
}

function authHeaders(): Record<string, string> {
	const headers: Record<string, string> = {};
	const token = session.authToken;
	if (token) {
		headers['Authorization'] = `Bearer ${token}`;
	}
	const tid = session.tenantId;
	if (tid) {
		headers['X-Tenant-Id'] = tid;
	}
	return headers;
}

async function throwForStatus(res: Response): Promise<never> {
	const text = await res.text();
	let parsed: { error?: string; errors?: Record<string, string[]> } = {};
	try {
		parsed = JSON.parse(text);
	} catch {
		// not JSON
	}

	if (res.status === 429) {
		throw new RateLimitError(res.headers, parsed.error);
	}
	if (res.status === 401) {
		throw new AuthError(parsed.error);
	}
	if (res.status === 422) {
		throw new ValidationError(parsed.errors ?? {}, parsed.error);
	}
	throw new ApiError(res.status, parsed.error ?? res.statusText);
}

export async function apiFetch<T>(path: string, options: ApiOptions = {}): Promise<T> {
	const { method = 'GET', body, headers = {} } = options;

	const res = await fetch(`${baseUrl()}${path}`, {
		method,
		headers: {
			'Content-Type': 'application/json',
			...authHeaders(),
			...headers
		},
		body: body != null ? JSON.stringify(body) : undefined
	});

	if (!res.ok) {
		return throwForStatus(res) as never;
	}

	return res.json() as Promise<T>;
}

export async function sseStream(path: string, options: SseOptions): Promise<void> {
	const { method = 'POST', body, onEvent, onError, onDone } = options;

	const res = await fetch(`${baseUrl()}${path}`, {
		method,
		headers: {
			'Content-Type': 'application/json',
			Accept: 'text/event-stream',
			...authHeaders()
		},
		body: body != null ? JSON.stringify(body) : undefined
	});

	if (!res.ok) {
		return throwForStatus(res) as never;
	}

	const reader = res.body?.getReader();
	if (!reader) {
		throw new ApiError(0, 'No response body');
	}

	const decoder = new TextDecoder();
	let buffer = '';

	try {
		while (true) {
			const { done, value } = await reader.read();
			if (done) break;

			buffer += decoder.decode(value, { stream: true });

			// Parse SSE lines: "data: {json}\n\n"
			const parts = buffer.split('\n\n');
			buffer = parts.pop() ?? '';

			for (const part of parts) {
				const line = part.trim();
				if (!line.startsWith('data: ')) continue;
				const json = line.slice(6);
				try {
					onEvent(JSON.parse(json));
				} catch (e) {
					onError?.(e instanceof Error ? e : new Error(String(e)));
				}
			}
		}
	} finally {
		reader.releaseLock();
	}

	onDone?.();
}
