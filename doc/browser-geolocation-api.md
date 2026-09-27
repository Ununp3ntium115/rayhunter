# Browser Geolocation API and GPS Configuration

Rayhunter's web interface can use your browser's Geolocation API to automatically populate GPS coordinates in the "Fixed coordinates" mode. This page explains how to use this feature, browser security requirements, and known limitations.

## Quick Start

When you open Rayhunter's configuration form and set the GPS Mode to "Fixed coordinates," you'll see a "Fill from Browser Location" button. Clicking this button will:

1. Request your browser's permission to access location data
2. Retrieve your current GPS coordinates
3. Automatically fill the latitude and longitude fields
4. Display any warnings (e.g., if coordinates are older than 5 minutes)

## Browser Security Requirements

The Geolocation API is only available in **secure contexts**. This means:

- **HTTPS URLs**: Always supported
- **localhost**: Supported (127.0.0.1, [::1])
- **Non-localhost HTTP**: NOT supported (blocked by browser security policy)

### Common Deployment Scenarios

**Default hotspot deployment (HTTP):**
The Rayhunter daemon usually runs on `http://192.168.1.1:8080` or similar, which is NOT a secure context. Geolocation will not work unless you:

1. **Port-forward to localhost** (Recommended):
   - SSH: `ssh -L 8080:192.168.1.1:8080 root@192.168.1.1`
   - ADB: `adb forward tcp:8080 tcp:8080`
   - Then access: `http://localhost:8080`

2. **Use HTTPS**: Deploy Rayhunter behind an HTTPS reverse proxy (Cloudflare Tunnel, nginx with a self-signed cert, etc.)

3. **Development**: If running `npm run dev` on your local machine, Rayhunter is already at `localhost:5173`, which is a secure context.

## Error Handling

The geolocation button handles several error cases:

| Error | Cause | Solution |
|-------|-------|----------|
| **Insecure Context** | Accessing Rayhunter over plain HTTP | Port-forward to localhost or use HTTPS |
| **Permission Denied** | You rejected the browser permission prompt | Click the button again and allow access |
| **Location Unavailable** | GPS is disabled or not available | Enable GPS on your device; some browsers require high-accuracy mode |
| **Timeout** | Request took more than 10 seconds | Try again; ensure GPS has a clear view of the sky |
| **Invalid Coordinates** | Location data failed validation | Rare; try again or manually enter coordinates |

## Coordinate Validation

Rayhunter validates all coordinates before accepting them:

- **Latitude**: Must be between -90 and 90 degrees (decimal)
- **Longitude**: Must be between -180 and 180 degrees (decimal)
- **Freshness**: Coordinates older than 5 minutes trigger a warning (but are still accepted)

Coordinates are never logged or exposed; only the latitude and longitude are stored.

## Browser Compatibility

| Browser | Desktop | Mobile |
|---------|---------|--------|
| Chrome/Chromium | ✅ | ✅ |
| Firefox | ✅ | ✅ |
| Safari | ✅ | ✅ |
| Edge | ✅ | ✅ |

**Note**: Firefox and Safari require explicit permission each time on some deployments.

## Privacy and Security

- Coordinates are requested **only with explicit user action** (clicking the button)
- No coordinates are automatically sent to Rayhunter's API; filling the form is purely local
- Coordinates are only stored in the fixed GPS configuration when you click "Apply and restart"
- Coordinates are never logged in Rayhunter's daemon logs
- Always use HTTPS or secure-context deployments for sensitive environments

## Known Limitations

1. **Mobile Background**: Geolocation may timeout if the browser is backgrounded
2. **Tab Visibility**: Some browsers throttle geolocation in background tabs
3. **GPS Warm-up**: First location requests may take 10+ seconds as GPS establishes a fix
4. **Network-only**: Devices without GPS hardware can attempt to use network-based location (less accurate)

## Advanced: Continuous Location Updates (Optional)

The current implementation uses single-shot location retrieval. For applications requiring continuous updates, the browser's `watchPosition()` API is available but has caveats:

- Consumes significant power (high-frequency GPS polling)
- Browser backgrounding will pause updates
- May require configurable polling intervals (not currently implemented)

If you need continuous location tracking, consider alternative approaches:
- Deploy a separate GPS logging service on the hotspot device
- Use Rayhunter's API endpoint mode to push coordinates programmatically
- Post-process recordings with external geolocation data

## Troubleshooting

**Button does not appear:**
- Ensure GPS Mode is set to "Fixed coordinates"
- Reload the page and try again

**"Insecure Context" error:**
- Check that you're accessing Rayhunter at `localhost` or via HTTPS
- Port-forward if accessing a remote hotspot device

**Permission prompt never appears:**
- Check your browser's location settings for the site
- Try in a private/incognito window (resets site permissions)
- Restart the browser if it's stuck

**Coordinates are stale:**
- Amber warning means coordinates are older than 5 minutes
- This is normal for initial GPS acquisition; refresh if needed
- You can still save stale coordinates if you prefer

**Timeout errors:**
- Ensure GPS is enabled on your device
- Move to an open area with clear sky view
- Retry with `enableHighAccuracy: true` (currently always on)

## Plain HTTP

Rayhunter serves its UI over plain `http://`, which browsers do not treat as a secure context, so `navigator.geolocation` is unavailable. The "Fill from Browser Location" button is disabled in that case and the coordinate fields stay editable. Use `localhost` (for example via port forwarding) or HTTPS to enable it.
