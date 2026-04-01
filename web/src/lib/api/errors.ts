export class ApiError extends Error {
	status: number;

	constructor(status: number, message: string) {
		super(message);
		this.name = 'ApiError';
		this.status = status;
	}
}

export class RateLimitError extends ApiError {
	retryAfter: number;
	limit: number;
	remaining: number;

	constructor(headers: Headers, message = 'Rate limit exceeded') {
		super(429, message);
		this.name = 'RateLimitError';
		this.retryAfter = parseInt(headers.get('Retry-After') ?? '0', 10);
		this.limit = parseInt(headers.get('X-RateLimit-Limit') ?? '0', 10);
		this.remaining = parseInt(headers.get('X-RateLimit-Remaining') ?? '0', 10);
	}
}

export class AuthError extends ApiError {
	constructor(message = 'Unauthorized') {
		super(401, message);
		this.name = 'AuthError';
	}
}

export class ValidationError extends ApiError {
	errors: Record<string, string[]>;

	constructor(errors: Record<string, string[]>, message = 'Validation failed') {
		super(422, message);
		this.name = 'ValidationError';
		this.errors = errors;
	}
}
