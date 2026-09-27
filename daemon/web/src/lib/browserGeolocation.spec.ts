import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { validate_coordinates, request_browser_location } from './browserGeolocation';

describe('Coordinate Validation', () => {
    it('accepts valid coordinates', () => {
        const result = validate_coordinates(37.7749, -122.4194);
        expect(result).toBeNull();
    });

    it('rejects non-finite latitude', () => {
        const result = validate_coordinates(NaN, -122.4194);
        expect(result).not.toBeNull();
        expect(result).toMatch(/valid.*number/i);
    });

    it('rejects non-finite longitude', () => {
        const result = validate_coordinates(37.7749, NaN);
        expect(result).not.toBeNull();
        expect(result).toMatch(/valid.*number/i);
    });

    it('rejects latitude > 90', () => {
        const result = validate_coordinates(91, -122.4194);
        expect(result).not.toBeNull();
        expect(result).toMatch(/[Ll]atitude/);
    });

    it('rejects latitude < -90', () => {
        const result = validate_coordinates(-91, -122.4194);
        expect(result).not.toBeNull();
        expect(result).toMatch(/[Ll]atitude/);
    });

    it('rejects longitude > 180', () => {
        const result = validate_coordinates(37.7749, 181);
        expect(result).not.toBeNull();
        expect(result).toMatch(/[Ll]ongitude/);
    });

    it('rejects longitude < -180', () => {
        const result = validate_coordinates(37.7749, -181);
        expect(result).not.toBeNull();
        expect(result).toMatch(/[Ll]ongitude/);
    });

    it('accepts boundary values', () => {
        expect(validate_coordinates(90, 180)).toBeNull();
        expect(validate_coordinates(-90, -180)).toBeNull();
        expect(validate_coordinates(0, 0)).toBeNull();
    });
});

describe('Browser Geolocation Request', () => {
    let mockGeolocation: any;
    let getCurrentPositionSpy: any;

    beforeEach(() => {
        vi.useFakeTimers();
        getCurrentPositionSpy = vi.fn();
        mockGeolocation = {
            getCurrentPosition: getCurrentPositionSpy,
        };
    });

    afterEach(() => {
        vi.useRealTimers();
        vi.clearAllMocks();
    });

    it('rejects when context is not secure', async () => {
        const result = await request_browser_location({
            geolocation: mockGeolocation,
            isSecureContext: false,
        });
        expect(result.ok).toBe(false);
        if (!result.ok) {
            expect(result.reason).toBe('insecure_context');
        }
    });

    it('rejects when geolocation is unavailable', async () => {
        const result = await request_browser_location({
            geolocation: undefined,
            isSecureContext: true,
        });
        expect(result.ok).toBe(false);
        if (!result.ok) {
            expect(result.reason).toBe('unsupported');
        }
    });

    it('rejects when permission is denied', async () => {
        getCurrentPositionSpy.mockImplementation((_success: any, error: any) => {
            error({ code: 1, message: 'User denied permission' });
        });

        const result = await request_browser_location({
            geolocation: mockGeolocation,
            isSecureContext: true,
        });
        expect(result.ok).toBe(false);
        if (!result.ok) {
            expect(result.reason).toBe('permission_denied');
        }
    });

    it('rejects when location is unavailable', async () => {
        getCurrentPositionSpy.mockImplementation((_success: any, error: any) => {
            error({ code: 2, message: 'Location is unavailable' });
        });

        const result = await request_browser_location({
            geolocation: mockGeolocation,
            isSecureContext: true,
        });
        expect(result.ok).toBe(false);
        if (!result.ok) {
            expect(result.reason).toBe('unavailable');
        }
    });

    it('handles timeout gracefully without hanging UI', async () => {
        getCurrentPositionSpy.mockImplementation((_success: any, error: any) => {
            error({ code: 3, message: 'Timeout' });
        });

        const result = await request_browser_location({
            geolocation: mockGeolocation,
            isSecureContext: true,
        });
        expect(result.ok).toBe(false);
        if (!result.ok) {
            expect(result.reason).toBe('timeout');
        }
    });

    it('handles fallback timeout (Promise.race)', async () => {
        // Never call either callback
        getCurrentPositionSpy.mockImplementation(() => {
            // Do nothing
        });

        const promise = request_browser_location({
            geolocation: mockGeolocation,
            isSecureContext: true,
        });

        // Fast-forward the timer past the timeout
        vi.advanceTimersByTime(11000);

        const result = await promise;
        expect(result.ok).toBe(false);
        if (!result.ok) {
            expect(result.reason).toBe('timeout');
        }
    });

    it('returns valid coordinates on success', async () => {
        const now = Date.now();
        getCurrentPositionSpy.mockImplementation((success: any) => {
            success({
                coords: {
                    latitude: 37.7749,
                    longitude: -122.4194,
                },
                timestamp: now,
            });
        });

        const result = await request_browser_location({
            geolocation: mockGeolocation,
            isSecureContext: true,
        });
        expect(result.ok).toBe(true);
        if (result.ok) {
            expect(result.latitude).toBe(37.7749);
            expect(result.longitude).toBe(-122.4194);
            expect(result.timestamp).toBe(now);
            expect(result.stale).toBe(false);
        }
    });

    it('marks coordinates as stale if older than 5 minutes', async () => {
        const fiveMinutesAgo = Date.now() - 5 * 60 * 1000 - 1000;
        getCurrentPositionSpy.mockImplementation((success: any) => {
            success({
                coords: {
                    latitude: 37.7749,
                    longitude: -122.4194,
                },
                timestamp: fiveMinutesAgo,
            });
        });

        const result = await request_browser_location({
            geolocation: mockGeolocation,
            isSecureContext: true,
        });
        expect(result.ok).toBe(true);
        if (result.ok) {
            expect(result.stale).toBe(true);
        }
    });

    it('rejects coordinates with invalid values', async () => {
        getCurrentPositionSpy.mockImplementation((success: any) => {
            success({
                coords: {
                    latitude: 91, // Invalid
                    longitude: -122.4194,
                },
                timestamp: Date.now(),
            });
        });

        const result = await request_browser_location({
            geolocation: mockGeolocation,
            isSecureContext: true,
        });
        expect(result.ok).toBe(false);
        if (!result.ok) {
            expect(result.reason).toBe('invalid_coordinates');
        }
    });
});
