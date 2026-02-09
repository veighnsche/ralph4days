# Sandbox Blocks ComfyUI Access

**Date:** 2026-02-09
**Issue:** ComfyUI connection failures during image generation
**Root Cause:** Claude Code sandbox network isolation

## The Problem

When attempting to generate discipline portrait images using `just gen-image`, the generation consistently failed with:

```
Generation failed: Failed to connect WebSocket: IO error: Connection refused (os error 111)
```

This error occurred even though ComfyUI was running and healthy.

## Root Cause: Sandbox Network Isolation

Claude Code runs commands in a sandbox using `bwrap --unshare-net`. This flag creates a separate network namespace, isolating the sandboxed process from the host network. Even though ComfyUI is running on `localhost:8188` on the host, the sandboxed process cannot reach it.

### Evidence

**Outside sandbox:**
```bash
$ ps aux | grep -i comfy | grep -v grep
vince    3265727 27.7 19.1 79309012 12544272 ?   Ssl  feb08 236:52 /home/vince/Projects/ComfyUI/.venv/bin/python main.py --listen

$ ss -tlnp | grep 8188
LISTEN 0      128          0.0.0.0:8188       0.0.0.0:*    users:(("python",pid=3265727,fd=53))

$ curl -s http://127.0.0.1:8188/system_stats | jq .system.comfyui_version
"0.10.0"
```

**Inside sandbox:**
```bash
$ curl http://127.0.0.1:8188/system_stats
curl: (7) Failed to connect to 127.0.0.1 port 8188 after 0 ms: Could not connect to server
```

## Failed Approaches

### 1. Preflight Check Removal

We removed the preflight check in `generate_discipline_image.rs` (lines 203-210) thinking it was an overly aggressive validation. This didn't help because the actual generation code also couldn't reach ComfyUI.

### 2. Retry Logic in `check_available`

We added retry logic, increased timeouts, and better error messages in `crates/ralph-external/src/comfy.rs`. This made diagnostics better but didn't solve the core issue of network isolation.

## Solution: Disable Sandbox

The user disabled the Claude Code sandbox to allow network access to ComfyUI. This is a configuration change in Claude Code settings, not in the project code.

## Impact on Image Generation Workflow

**Before fix:** All `just gen-image` commands failed with connection errors, making image generation impossible.

**After fix:** Image generation works normally, connecting to ComfyUI on `localhost:8188`.

## Lessons Learned

1. **Network isolation is invisible** - The sandbox's `--unshare-net` flag silently blocks all network access, including localhost. There's no indication in the error message that this is a sandbox issue.

2. **Preflight checks can be misleading** - The preflight check in `generate_discipline_image.rs` failed for the same reason the actual generation would fail (sandbox isolation), but we mistakenly thought the preflight check itself was the problem.

3. **curl/wget checks failed too** - Both curl checks and the Rust reqwest library failed identically when sandboxed, so there's no "better" detection method - the sandbox blocks all network access equally.

4. **Process visibility differs** - Inside the sandbox, `ps aux | grep comfy` returned no results. Outside the sandbox, it showed the running ComfyUI process. This is because the sandbox has its own PID namespace.

## Documentation Updates

Updated `crates/predefined-disciplines/IMAGE_REVIEW_CHECKLIST.md` lesson #7 to document that sandbox must be disabled for ComfyUI access.

## Solution Implementation

The preflight check was restored with **sandbox detection**:

```rust
let is_sandboxed = std::env::var("SANDBOX_RUNTIME").is_ok();

if !is_sandboxed {
    let status = ralph_external::check_comfy_available(&config).await;
    if !status.available {
        eprintln!("ComfyUI not available: {}", status.error.unwrap_or_default());
        eprintln!("Make sure ComfyUI is running at {}", config.api_url);
        std::process::exit(1);
    }
} else {
    eprintln!("NOTE: Running in sandbox - skipping ComfyUI preflight check");
    eprintln!("      If generation fails, sandbox network isolation may be the cause");
}
```

This approach:
- Checks for `SANDBOX_RUNTIME` env var (set by Claude Code sandbox)
- Runs preflight check only when NOT sandboxed
- Provides helpful message when sandboxed
- Allows generation to proceed and fail with actual error if ComfyUI is unreachable

## Files Modified

- `crates/predefined-disciplines/src/bin/generate_discipline_image.rs` - Added sandbox-aware preflight check
- `crates/ralph-external/src/comfy.rs` - Retry logic and better error messages (valuable for non-sandboxed runs)
- `crates/predefined-disciplines/IMAGE_REVIEW_CHECKLIST.md` - Updated lesson #7

## Related Documentation

- `.docs/041_COMFYUI_PREFLIGHT_CHECK_ROBUSTNESS.md` - Retry logic improvements (still useful, but didn't solve this issue)
- `.docs/042_IMAGE_GENERATION_REVIEW_CHECKLIST.md` - Workflow for generating images
