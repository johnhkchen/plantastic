import { describe, it, expect, beforeEach } from 'vitest';
import {
	checkIpRateLimit,
	checkSessionRateLimit,
	isOriginAllowed,
	corsHeaders,
	rateLimitHeaders,
	getLimit,
} from './index.js';
import type { RateLimitResult } from './index.js';

// --- getLimit ---

describe('getLimit', () => {
	it('returns parsed number for valid string', () => {
		expect(getLimit('42', 10)).toBe(42);
	});

	it('returns fallback for undefined', () => {
		expect(getLimit(undefined, 10)).toBe(10);
	});

	it('returns fallback for non-numeric string', () => {
		expect(getLimit('abc', 10)).toBe(10);
	});

	it('returns fallback for zero', () => {
		expect(getLimit('0', 10)).toBe(10);
	});

	it('returns fallback for negative', () => {
		expect(getLimit('-5', 10)).toBe(10);
	});
});

// --- checkIpRateLimit ---

describe('checkIpRateLimit', () => {
	it('allows requests under the limit', () => {
		const ip = `test-ip-${Date.now()}-under`;
		const result = checkIpRateLimit(ip, 5);
		expect(result.allowed).toBe(true);
		expect(result.remaining).toBe(4);
	});

	it('blocks requests at the limit', () => {
		const ip = `test-ip-${Date.now()}-at`;
		for (let i = 0; i < 3; i++) {
			checkIpRateLimit(ip, 3);
		}
		const result = checkIpRateLimit(ip, 3);
		expect(result.allowed).toBe(false);
		expect(result.remaining).toBe(0);
	});

	it('tracks remaining correctly', () => {
		const ip = `test-ip-${Date.now()}-remaining`;
		const r1 = checkIpRateLimit(ip, 5);
		expect(r1.remaining).toBe(4);
		const r2 = checkIpRateLimit(ip, 5);
		expect(r2.remaining).toBe(3);
		const r3 = checkIpRateLimit(ip, 5);
		expect(r3.remaining).toBe(2);
	});

	it('provides resetAt timestamp', () => {
		const ip = `test-ip-${Date.now()}-reset`;
		const result = checkIpRateLimit(ip, 5);
		expect(result.resetAt).toBeGreaterThan(Date.now());
		expect(result.resetAt).toBeLessThanOrEqual(Date.now() + 60_000);
	});
});

// --- checkSessionRateLimit ---

describe('checkSessionRateLimit', () => {
	it('allows requests under the limit', () => {
		const token = `session-${Date.now()}-under`;
		const result = checkSessionRateLimit(token, 10);
		expect(result.allowed).toBe(true);
		expect(result.remaining).toBe(9);
	});

	it('blocks requests at the limit', () => {
		const token = `session-${Date.now()}-at`;
		for (let i = 0; i < 5; i++) {
			checkSessionRateLimit(token, 5);
		}
		const result = checkSessionRateLimit(token, 5);
		expect(result.allowed).toBe(false);
		expect(result.remaining).toBe(0);
	});

	it('tracks independent sessions separately', () => {
		const token1 = `session-${Date.now()}-a`;
		const token2 = `session-${Date.now()}-b`;
		for (let i = 0; i < 3; i++) {
			checkSessionRateLimit(token1, 5);
		}
		const r1 = checkSessionRateLimit(token1, 5);
		const r2 = checkSessionRateLimit(token2, 5);
		expect(r1.remaining).toBe(1);
		expect(r2.remaining).toBe(4);
	});

	it('sets resetAt to 0 (lifetime counter)', () => {
		const token = `session-${Date.now()}-reset`;
		const result = checkSessionRateLimit(token, 10);
		expect(result.resetAt).toBe(0);
	});
});

// --- isOriginAllowed ---

describe('isOriginAllowed', () => {
	const makeEnv = (origin: string) =>
		({ ALLOWED_ORIGIN: origin }) as Parameters<typeof isOriginAllowed>[0];

	it('allows all origins with wildcard', () => {
		expect(isOriginAllowed(makeEnv('*'), 'https://example.com')).toBe(true);
		expect(isOriginAllowed(makeEnv('*'), null)).toBe(true);
	});

	it('allows matching origin', () => {
		expect(isOriginAllowed(makeEnv('https://app.plantastic.dev'), 'https://app.plantastic.dev')).toBe(true);
	});

	it('rejects non-matching origin', () => {
		expect(isOriginAllowed(makeEnv('https://app.plantastic.dev'), 'https://evil.com')).toBe(false);
	});

	it('rejects null origin when specific origin is configured', () => {
		expect(isOriginAllowed(makeEnv('https://app.plantastic.dev'), null)).toBe(false);
	});

	it('defaults to wildcard when ALLOWED_ORIGIN is empty', () => {
		expect(isOriginAllowed(makeEnv(''), 'https://example.com')).toBe(true);
	});
});

// --- corsHeaders ---

describe('corsHeaders', () => {
	const makeEnv = (origin: string) =>
		({ ALLOWED_ORIGIN: origin }) as Parameters<typeof corsHeaders>[0];

	it('returns wildcard origin for wildcard config', () => {
		const headers = corsHeaders(makeEnv('*'), 'https://example.com');
		expect(headers['Access-Control-Allow-Origin']).toBe('*');
	});

	it('reflects request origin for specific config', () => {
		const headers = corsHeaders(makeEnv('https://app.plantastic.dev'), 'https://app.plantastic.dev');
		expect(headers['Access-Control-Allow-Origin']).toBe('https://app.plantastic.dev');
	});

	it('returns empty object for rejected origin', () => {
		const headers = corsHeaders(makeEnv('https://app.plantastic.dev'), 'https://evil.com');
		expect(headers).toEqual({});
	});

	it('includes required methods', () => {
		const headers = corsHeaders(makeEnv('*'), null);
		expect(headers['Access-Control-Allow-Methods']).toContain('GET');
		expect(headers['Access-Control-Allow-Methods']).toContain('POST');
		expect(headers['Access-Control-Allow-Methods']).toContain('OPTIONS');
	});

	it('allows Authorization header', () => {
		const headers = corsHeaders(makeEnv('*'), null);
		expect(headers['Access-Control-Allow-Headers']).toContain('Authorization');
	});

	it('exposes rate limit headers', () => {
		const headers = corsHeaders(makeEnv('*'), null);
		expect(headers['Access-Control-Expose-Headers']).toContain('X-RateLimit-Limit');
		expect(headers['Access-Control-Expose-Headers']).toContain('X-RateLimit-Remaining');
		expect(headers['Access-Control-Expose-Headers']).toContain('X-RateLimit-Reset');
		expect(headers['Access-Control-Expose-Headers']).toContain('Retry-After');
	});
});

// --- rateLimitHeaders ---

describe('rateLimitHeaders', () => {
	it('returns limit and remaining', () => {
		const result: RateLimitResult = { allowed: true, remaining: 15, resetAt: 0 };
		const headers = rateLimitHeaders(result, 20);
		expect(headers['X-RateLimit-Limit']).toBe('20');
		expect(headers['X-RateLimit-Remaining']).toBe('15');
	});

	it('includes reset when resetAt > 0', () => {
		const resetAt = Date.now() + 30_000;
		const result: RateLimitResult = { allowed: true, remaining: 5, resetAt };
		const headers = rateLimitHeaders(result, 10);
		expect(headers['X-RateLimit-Reset']).toBe(Math.ceil(resetAt / 1000).toString());
	});

	it('omits reset when resetAt is 0', () => {
		const result: RateLimitResult = { allowed: true, remaining: 5, resetAt: 0 };
		const headers = rateLimitHeaders(result, 10);
		expect(headers['X-RateLimit-Reset']).toBeUndefined();
	});

	it('clamps remaining to 0', () => {
		const result: RateLimitResult = { allowed: false, remaining: -1, resetAt: 0 };
		const headers = rateLimitHeaders(result, 10);
		expect(headers['X-RateLimit-Remaining']).toBe('0');
	});
});
