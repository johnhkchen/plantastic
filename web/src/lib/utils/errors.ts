import { ApiError } from '$lib/api/errors';

/**
 * Classify an error into a human-readable message suitable for display.
 * Distinguishes network errors, server errors, and rate limits.
 */
export function friendlyError(error: unknown): string {
	if (error instanceof TypeError) {
		return "Couldn't reach the server. Check your connection and try again.";
	}

	if (error instanceof ApiError) {
		if (error.status === 429) {
			return 'Too many requests. Please wait a moment and try again.';
		}
		if (error.status >= 500) {
			return 'Something went wrong on our end. Please try again.';
		}
		// 4xx with a server-provided message — pass it through
		if (error.message && error.message !== 'ApiError') {
			return error.message;
		}
	}

	if (error instanceof Error && error.message) {
		return error.message;
	}

	return 'An unexpected error occurred. Please try again.';
}
