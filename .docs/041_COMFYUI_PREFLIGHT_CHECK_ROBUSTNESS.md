# ComfyUI Pre-flight Check Robustness Improvements

**Issue:** The `check_available` function consistently reported ComfyUI as offline even when it was running and working fine.

## Root Cause

The original implementation had several issues that made it overly aggressive and prone to false negatives:

1. **Timeout too short**: Single 2-second timeout for entire request lifecycle (DNS + connect + send + receive)
2. **No retry logic**: A single timeout/failure immediately marked ComfyUI as unavailable
3. **Poor diagnostics**: Error messages didn't distinguish between connection failures, timeouts, and HTTP errors

## Changes Made

### 1. Separate Connect and Total Timeouts

**Before:**
```rust
let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(2))  // Single timeout for everything
    .build()
```

**After:**
```rust
let client = reqwest::Client::builder()
    .connect_timeout(Duration::from_secs(5))  // Time to establish connection
    .timeout(Duration::from_secs(10))         // Total request time
    .build()
```

This prevents DNS resolution or connection establishment delays from consuming the entire timeout budget.

### 2. Retry Logic

The check now attempts the request **twice** with a 500ms delay between attempts:

```rust
for attempt in 1..=2 {
    match client.get(&url).send().await {
        Ok(resp) => { /* handle success/HTTP errors */ }
        Err(e) => {
            if attempt < 2 {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
}
```

This handles transient network issues and momentary ComfyUI slowdowns.

### 3. Better Error Diagnostics

Errors now include:
- **Error type classification** (timeout, connection failure, request error)
- **Attempt number** (1/2 or 2/2)
- **HTTP status details** for non-success responses (including canonical reason)
- **Full URL** in final error message for easier debugging

**Example error messages:**

- `"Timeout after 10s (attempt 1/2)"` - First attempt timed out
- `"Connection failed: Connection refused (attempt 2/2)"` - Both attempts failed to connect
- `"ComfyUI responded with HTTP 500: Internal Server Error"` - Server error
- `"Cannot reach ComfyUI at http://localhost:8188/system_stats: Connection failed..."`

### 4. More Lenient Success Criteria

The check now only fails if:
1. Both retry attempts fail with network/timeout errors, **OR**
2. ComfyUI responds with a non-2xx HTTP status code

## Testing

Updated test to verify error message format:

```rust
#[tokio::test]
async fn check_available_handles_unreachable() {
    let config = ComfyConfig {
        api_url: "http://localhost:99999".into(),
        default_workflow: "test.json".into(),
        timeout_secs: 10,
    };

    let status = check_available(&config).await;
    assert!(!status.available);
    assert!(status.error.is_some());
    let error_msg = status.error.unwrap();
    assert!(
        error_msg.contains("Cannot reach ComfyUI"),
        "Expected 'Cannot reach ComfyUI' in error, got: {error_msg}"
    );
}
```

## Impact

- **Fewer false negatives**: ComfyUI is less likely to be reported as offline when it's actually working
- **Better debugging**: Error messages help diagnose real issues (port wrong, ComfyUI actually down, etc.)
- **More resilient**: Handles transient network issues and momentary slowdowns

## Location

File: `crates/ralph-external/src/comfy.rs`
Function: `check_available(config: &ComfyConfig) -> ComfyStatus`

## Related Files

- Binary that uses this check: `crates/predefined-disciplines/src/bin/generate_discipline_image.rs` (lines 203-210)
- Config struct: `crates/ralph-external/src/config.rs`

## Notes

The timeout values (5s connect, 10s total) are intentionally generous to account for:
- Local ComfyUI instances that might be swapping models or under load
- Network latency for remote ComfyUI instances
- DNS resolution delays

If ComfyUI is truly offline or unreachable, the check will still fail within 20-21 seconds total (two attempts × 10s + 500ms delay).

## Update (2026-02-09)

These improvements made error messages more actionable but did NOT solve the core issue of ComfyUI appearing offline during image generation. The actual problem was **Claude Code's sandbox network isolation** (see `.docs/043_SANDBOX_BLOCKS_COMFYUI_ACCESS.md`). The sandbox's `--unshare-net` flag blocks all network access, including localhost.

The preflight check has been **restored with sandbox detection**. It now checks for the `SANDBOX_RUNTIME` environment variable:
- **Not sandboxed**: Runs preflight check with retry logic and exits with clear error if ComfyUI is unreachable
- **Sandboxed**: Skips preflight check and prints a warning that sandbox isolation may cause failures

This approach provides the best of both worlds:
- Helpful early feedback when running outside the sandbox
- No false negatives when running inside the sandbox
- Clear messaging about the sandbox limitation

The retry logic and better error messages remain valuable for diagnosing real ComfyUI failures when running in non-sandboxed environments.
