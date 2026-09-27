/**
 * Browser Geolocation API utility with validation and error handling.
 * Designed for node-env testing (uses numeric error codes, not class literals).
 */

type CoordinateValidationResult = string | null;
type LocationResult =
    | { ok: true; latitude: number; longitude: number; timestamp: number; stale: boolean }
    | {
          ok: false;
          reason:
              | 'insecure_context'
              | 'unsupported'
              | 'permission_denied'
              | 'unavailable'
              | 'timeout'
              | 'invalid_coordinates';
          message: string;
      };

/**
 * Validates that latitude is in [-90, 90] and longitude is in [-180, 180].
 * Returns error message if invalid, null if valid.
 * Does not log coordinates to avoid exposure.
 */
export function validate_coordinates(lat: unknown, lon: unknown): CoordinateValidationResult {
    // Check for non-finite values (NaN, Infinity, etc.)
    if (typeof lat !== 'number' || !Number.isFinite(lat)) {
        return 'Latitude must be a valid number';
    }
    if (typeof lon !== 'number' || !Number.isFinite(lon)) {
        return 'Longitude must be a valid number';
    }

    // Check ranges
    if (lat < -90 || lat > 90) {
        return 'Latitude must be between -90 and 90 degrees';
    }
    if (lon < -180 || lon > 180) {
        return 'Longitude must be between -180 and 180 degrees';
    }

    return null;
}

/**
 * Requests browser geolocation with proper error handling.
 * Returns a discriminated union with ok: true | false.
 * Stale coordinates (>5 min old) still return ok:true with stale:true flag.
 */
export async function request_browser_location(deps: {
    geolocation: Geolocation | undefined;
    isSecureContext: boolean;
}): Promise<LocationResult> {
    // Check secure context first
    if (!deps.isSecureContext) {
        return {
            ok: false,
            reason: 'insecure_context',
            message:
                'Geolocation requires a secure context (HTTPS or localhost). Port-forward to localhost:8080 or use HTTPS.',
        };
    }

    // Check browser support
    if (!deps.geolocation) {
        return {
            ok: false,
            reason: 'unsupported',
            message: 'Geolocation API is not available in this browser or device.',
        };
    }

    // Type narrowing: geolocation is now guaranteed to be non-undefined
    const geolocation = deps.geolocation;

    return new Promise((resolve) => {
        const timeoutHandle = setTimeout(() => {
            resolve({
                ok: false,
                reason: 'timeout',
                message:
                    'Geolocation request timed out after 10 seconds. Check permissions and ensure location is available.',
            });
        }, 10000);

        geolocation.getCurrentPosition(
            (position) => {
                clearTimeout(timeoutHandle);

                const { latitude, longitude } = position.coords;
                const validationError = validate_coordinates(latitude, longitude);

                if (validationError) {
                    resolve({
                        ok: false,
                        reason: 'invalid_coordinates',
                        message: validationError,
                    });
                    return;
                }

                // Check if coordinates are stale (older than 5 minutes)
                const fiveMinutesMs = 5 * 60 * 1000;
                const now = Date.now();
                const stale = now - position.timestamp > fiveMinutesMs;

                resolve({
                    ok: true,
                    latitude,
                    longitude,
                    timestamp: position.timestamp,
                    stale,
                });
            },
            (error) => {
                clearTimeout(timeoutHandle);

                let reason:
                    | 'insecure_context'
                    | 'unsupported'
                    | 'permission_denied'
                    | 'unavailable'
                    | 'timeout'
                    | 'invalid_coordinates' = 'unavailable';
                let message: string = 'An unknown geolocation error occurred.';

                // Use numeric codes (1, 2, 3) not class literals (not available in node env)
                switch (error.code) {
                    case 1: // PERMISSION_DENIED
                        reason = 'permission_denied';
                        message = 'Location permission was denied.';
                        break;
                    case 2: // POSITION_UNAVAILABLE
                        reason = 'unavailable';
                        message =
                            'Location could not be determined. Check if GPS is available on this device.';
                        break;
                    case 3: // TIMEOUT
                        reason = 'timeout';
                        message =
                            'Geolocation request timed out. Ensure location services are enabled.';
                        break;
                }

                resolve({
                    ok: false,
                    reason,
                    message,
                });
            },
            {
                timeout: 10000,
                enableHighAccuracy: true,
            }
        );
    });
}
