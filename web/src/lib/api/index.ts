export { ApiError, AuthError, RateLimitError, ValidationError } from './errors';
export type { ApiOptions, SseOptions } from './client';

import { apiFetch as realApiFetch, sseStream as realSseStream } from './client';
import { mockApiFetch, mockSseStream } from './mock';

const useMock = import.meta.env.VITE_MOCK_API === 'true';

export const apiFetch = useMock ? mockApiFetch : realApiFetch;
export const sseStream = useMock ? mockSseStream : realSseStream;
