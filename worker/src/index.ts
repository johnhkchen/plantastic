interface Env {
	BACKEND_URL: string;
	ALLOWED_ORIGIN: string;
	RATE_LIMIT_IP_MAX?: string;
	RATE_LIMIT_SESSION_MAX?: string;
}

// --- Rate Limiting ---

export interface RateLimitResult {
	allowed: boolean;
	remaining: number;
	/** Unix ms when the oldest entry in the window expires (0 if not applicable) */
	resetAt: number;
}

const RATE_LIMIT_WINDOW_MS = 60_000;

export function getLimit(val: string | undefined, fallback: number): number {
	if (!val) return fallback;
	const n = parseInt(val, 10);
	return Number.isFinite(n) && n > 0 ? n : fallback;
}

// Per-IP sliding window
const ipTimestamps = new Map<string, number[]>();

export function checkIpRateLimit(ip: string, max: number): RateLimitResult {
	const now = Date.now();
	const cutoff = now - RATE_LIMIT_WINDOW_MS;

	let timestamps = ipTimestamps.get(ip);
	if (!timestamps) {
		timestamps = [];
		ipTimestamps.set(ip, timestamps);
	}

	// Remove expired timestamps
	const valid = timestamps.filter((t) => t > cutoff);

	if (valid.length >= max) {
		ipTimestamps.set(ip, valid);
		const resetAt = valid[0] + RATE_LIMIT_WINDOW_MS;
		return { allowed: false, remaining: 0, resetAt };
	}

	valid.push(now);
	ipTimestamps.set(ip, valid);

	// Periodic cleanup: remove IPs with no recent requests
	if (ipTimestamps.size > 10_000) {
		for (const [key, ts] of ipTimestamps) {
			if (ts.every((t) => t <= cutoff)) {
				ipTimestamps.delete(key);
			}
		}
	}

	const resetAt = valid[0] + RATE_LIMIT_WINDOW_MS;
	return { allowed: true, remaining: max - valid.length, resetAt };
}

// Per-session lifetime counter
const sessionCounts = new Map<string, number>();

export function checkSessionRateLimit(token: string, max: number): RateLimitResult {
	const count = (sessionCounts.get(token) ?? 0) + 1;
	sessionCounts.set(token, count);

	// Periodic cleanup: remove exhausted sessions
	if (sessionCounts.size > 10_000) {
		for (const [key, c] of sessionCounts) {
			if (c >= max) {
				sessionCounts.delete(key);
			}
		}
	}

	return {
		allowed: count <= max,
		remaining: Math.max(0, max - count),
		resetAt: 0,
	};
}

// --- Headers ---

export function rateLimitHeaders(result: RateLimitResult, max: number): Record<string, string> {
	const headers: Record<string, string> = {
		'X-RateLimit-Limit': max.toString(),
		'X-RateLimit-Remaining': Math.max(0, result.remaining).toString(),
	};
	if (result.resetAt > 0) {
		headers['X-RateLimit-Reset'] = Math.ceil(result.resetAt / 1000).toString();
	}
	return headers;
}

export function isOriginAllowed(env: Env, requestOrigin: string | null): boolean {
	const allowed = env.ALLOWED_ORIGIN || '*';
	if (allowed === '*') return true;
	return requestOrigin === allowed;
}

export function corsHeaders(env: Env, requestOrigin: string | null): Record<string, string> {
	if (!isOriginAllowed(env, requestOrigin)) {
		return {};
	}
	const allowed = env.ALLOWED_ORIGIN || '*';
	return {
		'Access-Control-Allow-Origin': allowed === '*' ? '*' : requestOrigin!,
		'Access-Control-Allow-Methods': 'GET, POST, PUT, PATCH, DELETE, OPTIONS',
		'Access-Control-Allow-Headers': 'Content-Type, Authorization',
		'Access-Control-Expose-Headers': 'X-RateLimit-Limit, X-RateLimit-Remaining, X-RateLimit-Reset, Retry-After',
		'Access-Control-Max-Age': '86400',
	};
}

function jsonError(
	message: string,
	status: number,
	env: Env,
	requestOrigin: string | null,
	extra?: Record<string, string>,
): Response {
	return new Response(JSON.stringify({ error: message }), {
		status,
		headers: {
			'Content-Type': 'application/json',
			...corsHeaders(env, requestOrigin),
			...extra,
		},
	});
}

// --- Proxy ---

async function proxyToBackend(
	request: Request,
	env: Env,
	requestOrigin: string | null,
	extraHeaders: Record<string, string>,
): Promise<Response> {
	const url = new URL(request.url);
	const targetUrl = env.BACKEND_URL.replace(/\/+$/, '') + url.pathname;

	// Forward safe headers to backend
	const forwardHeaders = new Headers();
	const contentType = request.headers.get('Content-Type');
	if (contentType) {
		forwardHeaders.set('Content-Type', contentType);
	}
	const authorization = request.headers.get('Authorization');
	if (authorization) {
		forwardHeaders.set('Authorization', authorization);
	}
	const tenantId = request.headers.get('X-Tenant-Id');
	if (tenantId) {
		forwardHeaders.set('X-Tenant-Id', tenantId);
	}

	let backendResponse: Response;
	try {
		backendResponse = await fetch(targetUrl, {
			method: request.method,
			headers: forwardHeaders,
			body: request.method !== 'GET' ? request.body : undefined,
		});
	} catch {
		return jsonError('Backend unavailable', 502, env, requestOrigin);
	}

	// Build response headers: backend headers + CORS + rate limit
	const responseHeaders = new Headers(backendResponse.headers);
	for (const [key, value] of Object.entries(corsHeaders(env, requestOrigin))) {
		responseHeaders.set(key, value);
	}
	for (const [key, value] of Object.entries(extraHeaders)) {
		responseHeaders.set(key, value);
	}

	// Pass through the response body as a stream (no buffering)
	return new Response(backendResponse.body, {
		status: backendResponse.status,
		headers: responseHeaders,
	});
}

// --- Main Handler ---

export default {
	async fetch(request: Request, env: Env): Promise<Response> {
		const requestOrigin = request.headers.get('Origin');

		// CORS preflight
		if (request.method === 'OPTIONS') {
			if (!isOriginAllowed(env, requestOrigin)) {
				return new Response(null, { status: 403 });
			}
			return new Response(null, {
				status: 204,
				headers: corsHeaders(env, requestOrigin),
			});
		}

		const url = new URL(request.url);

		// Route matching: /api/* and /health
		if (!url.pathname.startsWith('/api/') && url.pathname !== '/health') {
			return jsonError('Not found', 404, env, requestOrigin);
		}

		// Only allow known methods
		const allowedMethods = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE'];
		if (!allowedMethods.includes(request.method)) {
			return jsonError('Method not allowed', 405, env, requestOrigin);
		}

		// Parse configurable limits
		const ipMax = getLimit(env.RATE_LIMIT_IP_MAX, 60);
		const sessionMax = getLimit(env.RATE_LIMIT_SESSION_MAX, 200);

		// Rate limiting by IP
		const ip = request.headers.get('cf-connecting-ip') || 'unknown';
		const ipResult = checkIpRateLimit(ip, ipMax);
		const rlHeaders = rateLimitHeaders(ipResult, ipMax);

		if (!ipResult.allowed) {
			const retryAfter = Math.max(1, Math.ceil((ipResult.resetAt - Date.now()) / 1000));
			return jsonError('Rate limit exceeded', 429, env, requestOrigin, {
				...rlHeaders,
				'Retry-After': retryAfter.toString(),
			});
		}

		// Rate limiting by session (using Authorization token as session identifier)
		const authToken = request.headers.get('Authorization');
		if (authToken) {
			const sessionResult = checkSessionRateLimit(authToken, sessionMax);
			if (!sessionResult.allowed) {
				return jsonError('Session rate limit exceeded', 429, env, requestOrigin, {
					...rlHeaders,
					'Retry-After': '0',
				});
			}
		}

		// Check backend is configured
		if (!env.BACKEND_URL) {
			return jsonError('Backend not configured', 502, env, requestOrigin);
		}

		// Proxy to backend with rate limit headers
		return proxyToBackend(request, env, requestOrigin, rlHeaders);
	},
} satisfies ExportedHandler<Env>;
