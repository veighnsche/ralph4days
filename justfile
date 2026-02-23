# Ralph Loop development commands
# Run with: just <command>
# Install just: cargo install just

set shell := ["bash", "-cu"]
mock_dir := env_var_or_default('RALPH_MOCK_DIR', '/tmp/ralph4days-mock')

# Generate a discipline portrait: just gen-image 02 00 [--test|--half] [--ratio W H|--ratio-portrait] [--mp N]
gen-image STACK DISCIPLINE *FLAGS:
    cargo run -p catalog-disciplines --bin generate-discipline-image -- {{STACK}} {{DISCIPLINE}} {{FLAGS}}

# Default recipe: show available commands
default:
    @just --list

# === Development ===

# Start development server (frontend + backend hot reload)
dev:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "$(uname -s)" = "Linux" ]; then
        WEBKIT_DISABLE_DMABUF_RENDERER=1 bun tauri dev
    else
        bun tauri dev
    fi

# Start frontend dev server only
dev-frontend:
    bun dev

# Open the iOS Xcode project for manual run/signing/debug
dev-ios-open:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "$(uname -s)" != "Darwin" ]; then
        echo "❌ dev-ios-open requires macOS (Darwin). Current OS: $(uname -s)"
        exit 1
    fi
    bun tauri ios dev --open

# Run iOS app on a simulator/device by display name (example: just dev-ios "iPhone 17 Pro")
dev-ios DEVICE:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "$(uname -s)" != "Darwin" ]; then
        echo "❌ dev-ios requires macOS (Darwin). Current OS: $(uname -s)"
        exit 1
    fi
    if [ -z "${TAURI_DEV_HOST:-}" ]; then
        for iface in en0 en1; do
            candidate="$(ipconfig getifaddr "$iface" 2>/dev/null || true)"
            if [ -n "${candidate}" ]; then
                TAURI_DEV_HOST="${candidate}"
                break
            fi
        done
        if [ -z "${TAURI_DEV_HOST:-}" ]; then
            echo "❌ Failed to resolve TAURI_DEV_HOST from en0/en1. Set TAURI_DEV_HOST manually."
            exit 1
        fi
    fi
    export TAURI_DEV_HOST
    echo "==> Using TAURI_DEV_HOST=${TAURI_DEV_HOST}"
    simulator_udid="$(xcrun simctl list devices available | awk -F '[()]' '/{{DEVICE}} \(/ { print $2; exit }')"
    if [ -n "${simulator_udid}" ]; then
        echo "==> Ensuring simulator '{{DEVICE}}' is booted (${simulator_udid})"
        xcrun simctl boot "${simulator_udid}" >/dev/null 2>&1 || true
        xcrun simctl bootstatus "${simulator_udid}" -b
    fi
    bun tauri ios dev "{{DEVICE}}"

# Build iOS simulator debug bundle (fast validation gate for mobile runtime linkage)
build-ios-sim:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "$(uname -s)" != "Darwin" ]; then
        echo "❌ build-ios-sim requires macOS (Darwin). Current OS: $(uname -s)"
        exit 1
    fi
    bun tauri ios build --debug --target aarch64-sim --ci

# Start Storybook dev server
storybook:
    bun storybook

# Build static Storybook
build-storybook:
    bun build-storybook

# Start development server with a mock project (skips project picker)
dev-mock FIXTURE:
    #!/usr/bin/env bash
    MOCK_DIR="{{mock_dir}}"

    # Ensure mock directory exists and has at least one mock project
    if [ ! -d "$MOCK_DIR" ]; then
        echo "Mock directory not found at $MOCK_DIR. Creating from fixtures..."
        just reset-mock
    else
        shopt -s nullglob
        existing_projects=("$MOCK_DIR"/*/)
        if [ ${#existing_projects[@]} -eq 0 ]; then
            echo "No mock projects found in $MOCK_DIR. Creating from fixtures..."
            just reset-mock
        fi
    fi

    # Try exact match first
    if [ -d "$MOCK_DIR/{{FIXTURE}}" ]; then
        PROJECT_DIR="$MOCK_DIR/{{FIXTURE}}"
    else
        # Try prefix match (e.g., "01" matches "01-desktop-blank")
        MATCHES=("$MOCK_DIR"/{{FIXTURE}}*/)
        if [ ${#MATCHES[@]} -eq 1 ] && [ -d "${MATCHES[0]}" ]; then
            PROJECT_DIR="${MATCHES[0]}"
            echo "✓ Found: $(basename "$PROJECT_DIR")"
        elif [ ${#MATCHES[@]} -gt 1 ]; then
            echo "❌ Multiple matches found for '{{FIXTURE}}':"
            for m in "${MATCHES[@]}"; do
                echo "  - $(basename "$m")"
            done
            exit 1
        else
            echo "❌ No mock project found matching '{{FIXTURE}}'"
            echo "Available projects:"
            ls -1 "$MOCK_DIR"
            exit 1
        fi
    fi

    if [ "$(uname -s)" = "Linux" ]; then
        WEBKIT_DISABLE_DMABUF_RENDERER=1 bun tauri dev -- -- --project "$PROJECT_DIR"
    else
        bun tauri dev -- -- --project "$PROJECT_DIR"
    fi

# Run cargo check (fast compilation check)
check:
    cargo check --manifest-path src-tauri/Cargo.toml

# Mobile compile/dependency gate (defaults to Android ARM64 target)
check-mobile TARGET="aarch64-linux-android":
    #!/usr/bin/env bash
    set -euo pipefail

    if ! rustup target list --installed | grep -qx "{{TARGET}}"; then
        echo "❌ Rust target not installed: {{TARGET}}"
        echo "Run: rustup target add {{TARGET}}"
        exit 1
    fi

    bunx tsc --noEmit
    cargo check --manifest-path src-tauri/Cargo.toml --target "{{TARGET}}" --lib

    forbidden="$(cargo tree --manifest-path src-tauri/Cargo.toml --target "{{TARGET}}" -e normal | rg 'data-sqlite|ai-prompt-builder|service(-|$)|portable-pty|axum' || true)"
    if [ -n "${forbidden}" ]; then
        echo "❌ Forbidden desktop backend crates detected in mobile dependency graph:"
        echo "${forbidden}"
        exit 1
    fi

    echo "✓ Mobile dependency gate passed for {{TARGET}}"

# Run lints (Rust + TypeScript)
lint:
    cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
    bunx oxlint src
    bunx biome lint src

# Fix linting issues automatically
lint-fix:
    bunx biome lint --write src

# Format all code (Rust + TypeScript)
fmt:
    cargo fmt --manifest-path src-tauri/Cargo.toml
    bunx biome format --write src

# Check formatting without writing
fmt-check:
    cargo fmt --manifest-path src-tauri/Cargo.toml --check
    bunx biome format src

# Run all checks (lint + format)
check-all: lint fmt-check

# Frontend compile/type gate
frontend-typecheck:
    bunx tsc --noEmit

# Quick correctness gate (lint + format + frontend typecheck + generated types + contract drift tests)
verify: check-all frontend-typecheck types-check contract-tests

# Contract-only gate that is CI-friendly (no GUI runtime).
verify-contract: types-check contract-tests
    bun test:run src/lib/terminal/terminalBridgeContract.test.ts src/lib/tauri/eventsContract.test.ts src/lib/tauri/tauriImportBoundary.test.ts

# Swap readiness gate: contract stability + ralphd WS smoke/parity tests.
verify-swap: verify-contract
    cargo test -p ralphd

# Minimal test surface that catches IPC/type/contract drift without running the full test suite.
contract-tests:
    cargo test -p core-contracts
    cargo test --manifest-path src-tauri/Cargo.toml --test invoke_command_list_contract_test

# Slower gate (adds full unit tests)
verify-full: verify test

# === Testing ===

# Run all tests
test: test-rust test-frontend

# Run Rust tests
test-rust:
    cargo test --manifest-path src-tauri/Cargo.toml
    cargo test -p core-contracts

# Run backend terminal-bridge test suite only
test-terminal-bridge-backend:
    cargo test -p service-terminal terminal_bridge
    cargo test -p service-terminal terminal::manager::tests

# Run frontend unit tests
test-frontend:
    bun test:run

# Run Tauri desktop e2e tests against a prepared mock project
e2e-preflight:
    bun run preflight:e2e

test-e2e FIXTURE="04-desktop-dev":
	bun run audit:no-playwright
	PROJECT_DIR="{{mock_dir}}/{{FIXTURE}}" && [ -d "$PROJECT_DIR" ] || (echo "❌ Mock project not found: $PROJECT_DIR"; echo "Run: just reset-mock"; exit 1) && [ -d "$PROJECT_DIR/.ralph" ] || (echo "❌ Not an initialized Ralph project: $PROJECT_DIR/.ralph"; echo "Run: just reset-mock"; exit 1) && RALPH_E2E_PROJECT="$PROJECT_DIR" just e2e-preflight && RALPH_E2E_PROJECT="$PROJECT_DIR" bun x wdio run wdio.conf.js

# Run terminal e2e smoke only
test-e2e-terminal FIXTURE="04-desktop-dev":
	bun run audit:no-playwright
	PROJECT_DIR="{{mock_dir}}/{{FIXTURE}}" && [ -d "$PROJECT_DIR" ] || (echo "❌ Mock project not found: $PROJECT_DIR"; echo "Run: just reset-mock"; exit 1) && [ -d "$PROJECT_DIR/.ralph" ] || (echo "❌ Not an initialized Ralph project: $PROJECT_DIR/.ralph"; echo "Run: just reset-mock"; exit 1) && RALPH_E2E_PROJECT="$PROJECT_DIR" just e2e-preflight && RALPH_E2E_PROJECT="$PROJECT_DIR" bun x wdio run wdio.conf.js --spec e2e-tauri/terminal.spec.js

# Run remote-ssh mobile-panel e2e harness and capture screenshots (desktop webdriver runtime)
test-e2e-remote-ssh FIXTURE="04-desktop-dev":
	bun run audit:no-playwright
	PROJECT_DIR="{{mock_dir}}/{{FIXTURE}}" && [ -d "$PROJECT_DIR" ] || (echo "❌ Mock project not found: $PROJECT_DIR"; echo "Run: just reset-mock"; exit 1) && [ -d "$PROJECT_DIR/.ralph" ] || (echo "❌ Not an initialized Ralph project: $PROJECT_DIR/.ralph"; echo "Run: just reset-mock"; exit 1) && RALPH_E2E_PROJECT="$PROJECT_DIR" just e2e-preflight && RALPH_E2E_PROJECT="$PROJECT_DIR" RALPH_E2E_FORCE_REMOTE_PANEL=1 bun x wdio run wdio.conf.js --spec e2e-tauri/remote-ssh-mobile.spec.js

# Run iOS remote-ssh UI e2e harness against the real Tauri IPC runtime (Appium + screenshots)
test-ios-e2e-remote-ssh DEVICE="iPhone 17 Pro":
	#!/usr/bin/env bash
	set -euo pipefail
	if [ "$(uname -s)" != "Darwin" ]; then
		echo "❌ test-ios-e2e-remote-ssh requires macOS (Darwin). Current OS: $(uname -s)"
		exit 1
	fi
	RALPH_IOS_E2E_DEVICE="{{DEVICE}}" bash scripts/run-ios-e2e-remote-ssh.sh

# Run iOS remote-ssh UI harness with a macOS SSH target pre-provisioned (fixture + ralphd)
test-ios-e2e-remote-ssh-macos-target DEVICE="iPhone 17 Pro":
	#!/usr/bin/env bash
	set -euo pipefail
	if [ "$(uname -s)" != "Darwin" ]; then
		echo "❌ test-ios-e2e-remote-ssh-macos-target requires macOS (Darwin). Current OS: $(uname -s)"
		exit 1
	fi
	RALPH_IOS_E2E_DEVICE="{{DEVICE}}" bash scripts/run-ios-e2e-remote-ssh-macos-target.sh

# Run iOS remote-ssh UI e2e harness using XCTest runner (fallback path)
test-ios-e2e-remote-ssh-xctest DEVICE="iPhone 17 Pro":
	#!/usr/bin/env bash
	set -euo pipefail
	if [ "$(uname -s)" != "Darwin" ]; then
		echo "❌ test-ios-e2e-remote-ssh-xctest requires macOS (Darwin). Current OS: $(uname -s)"
		exit 1
	fi
	RALPH_IOS_E2E_DEVICE="{{DEVICE}}" RALPH_IOS_E2E_RUNNER=xctest bash scripts/run-ios-e2e-remote-ssh.sh

# Verify active e2e runtime surface has no forbidden browser-e2e framework references
audit-no-playwright:
    bun run audit:no-playwright

# === Building ===

# Build release desktop binary for the current host platform
build:
    NO_STRIP=1 bun tauri build

# Build debug binary (faster compilation)
build-debug:
    bun tauri build --debug

# Build frontend only
build-frontend:
    bun build

# Clean build artifacts
clean:
    cargo clean --manifest-path src-tauri/Cargo.toml
    rm -rf dist/

# === Release ===

# Build all Linux packages (deb, rpm, appimage)
release-linux:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "$(uname -s)" != "Linux" ]; then
        echo "❌ release-linux requires Linux. Current OS: $(uname -s)"
        exit 1
    fi
    NO_STRIP=1 bun tauri build --bundles deb,rpm,appimage

# Build macOS bundles (.app + .dmg)
release-macos:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "$(uname -s)" != "Darwin" ]; then
        echo "❌ release-macos requires macOS (Darwin). Current OS: $(uname -s)"
        exit 1
    fi
    NO_STRIP=1 bun tauri build --bundles app,dmg

# Build macOS bundles tuned for the current Apple Silicon CPU (local distribution only)
release-macos-native:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "$(uname -s)" != "Darwin" ]; then
        echo "❌ release-macos-native requires macOS (Darwin). Current OS: $(uname -s)"
        exit 1
    fi
    RUSTFLAGS='-C target-cpu=native' NO_STRIP=1 bun tauri build --bundles app,dmg

# === Utilities ===

# Temporary: best-effort model discovery for Codex CLI
get-codex-models:
    cargo run --manifest-path src-tauri/Cargo.toml --bin provider-models -- codex

# Temporary: best-effort model discovery for Claude CLI
get-claude-models:
    cargo run --manifest-path src-tauri/Cargo.toml --bin provider-models -- claude

# Check if mold linker is installed
check-mold:
    #!/usr/bin/env bash
    if [ "$(uname -s)" = "Linux" ]; then
        which mold >/dev/null && echo "✓ mold linker installed" || echo "✗ mold not found - install with your distro package manager"
    else
        echo "ℹ mold linker check is Linux-specific; skipped on $(uname -s)"
    fi

# Show system info relevant to development
sysinfo:
    #!/usr/bin/env bash
    set -euo pipefail

    echo "=== OS ==="
    uname -a

    if [ "$(uname -s)" = "Darwin" ]; then
        echo
        echo "=== CPU ==="
        sysctl -n machdep.cpu.brand_string 2>/dev/null || true
        echo "arch: $(uname -m)"

        echo
        echo "=== Memory ==="
        sysctl -n hw.memsize | awk '{printf "ram_bytes: %s\n", $1}'
        vm_stat | head -n 5

        echo
        echo "=== GPU ==="
        system_profiler SPDisplaysDataType 2>/dev/null | grep -E 'Chipset Model|Vendor|Metal' || true
    else
        echo
        echo "=== CPU ==="
        lscpu | grep "Model name"

        echo
        echo "=== Memory ==="
        free -h | head -2

        echo
        echo "=== GPU ==="
        nvidia-smi --query-gpu=name,memory.total --format=csv,noheader 2>/dev/null || echo "No NVIDIA GPU"
    fi

    echo
    echo "=== Rust ==="
    rustc --version

    echo
    echo "=== Node ==="
    node --version

# Open project in VS Code
code:
    code .

# Watch for file changes and run tests
watch-test:
    cargo watch --manifest-path src-tauri/Cargo.toml -x test

# Generate TypeScript types from Rust via ts-rs (single file, no barrel)
types:
    #!/usr/bin/env bash
    set -euo pipefail
    export LC_ALL=C

    # Clean old exports.
    rm -rf target/ts-bindings
    find crates src-tauri src-daemon -type d -path '*/target/ts-bindings' -prune -exec rm -rf {} +

    # Run ts-rs export tests. Depending on ts-rs version/config, outputs may land in the workspace
    # `target/ts-bindings` or per-crate `*/target/ts-bindings`. We normalize to the workspace path.
    cargo test --workspace -- export_bindings

    shopt -s nullglob
    root_outputs=(target/ts-bindings/*.ts)
    if [ "${#root_outputs[@]}" -eq 0 ]; then
        ts_files=()
        while IFS= read -r -d '' file; do
            ts_files+=("$file")
        done < <(find crates src-tauri src-daemon -type f -path '*/target/ts-bindings/*.ts' -print0)
        if [ "${#ts_files[@]}" -eq 0 ]; then
            echo '❌ ts-rs produced no bindings under target/ts-bindings/*.ts or */target/ts-bindings/*.ts'
            exit 1
        fi

        duplicates="$(printf '%s\n' "${ts_files[@]}" | xargs -n1 basename | sort | uniq -d || true)"
        if [ -n "${duplicates}" ]; then
            echo '❌ Duplicate ts-rs output filenames detected (type ownership violation):'
            echo "${duplicates}"
            exit 1
        fi

        mkdir -p target/ts-bindings
        for f in "${ts_files[@]}"; do
            cp "${f}" "target/ts-bindings/$(basename "${f}")"
        done
    fi

    echo '// Auto-generated by ts-rs — do not edit. Regenerate: just types' > src/types/generated.ts
    for f in target/ts-bindings/*.ts; do
        grep -hvE '^(import type|//)' "${f}" | sed '/^\/\*\*/,/\*\//d' | grep -v '^$' >> src/types/generated.ts
    done

    bunx biome check --write --unsafe src/types/generated.ts

# Check if generated types are up to date
types-check:
    #!/usr/bin/env bash
    just types
    if ! git diff --quiet src/types/generated.ts; then
        echo "❌ Generated types are stale. Run 'just types' and commit."
        exit 1
    fi

# === Mock Test Data ===

# Reset mock directory from fixtures (copies fixtures → $RALPH_MOCK_DIR or /tmp/ralph4days-mock, makes .ralph visible)
reset-mock:
    @bash scripts/reset-mock.sh

# Rebuild Tauri backend, regenerate all fixtures, then copy fixtures -> mock
refresh-tauri-fixtures-mock:
    cargo build --manifest-path src-tauri/Cargo.toml
    cargo test --manifest-path src-tauri/Cargo.toml --test generate_fixtures generate_all_fixtures -- --ignored --nocapture --test-threads=1
    bash scripts/verify-fixtures.sh
    just reset-mock

# List available mock projects
list-mock:
    #!/usr/bin/env bash
    MOCK_DIR="{{mock_dir}}"
    if [ ! -d "$MOCK_DIR" ]; then
        echo "No mock directory found at $MOCK_DIR. Run 'just reset-mock' first."
        exit 1
    fi
    shopt -s nullglob
    projects=("$MOCK_DIR"/*/)
    if [ ${#projects[@]} -eq 0 ]; then
        echo "No mock projects found in $MOCK_DIR. Run 'just reset-mock' first."
        exit 1
    fi
    echo "Available mock projects in $MOCK_DIR:"
    for f in "${projects[@]}"; do
        name=$(basename "$f")
        db="${f}.ralph/db/ralph.db"
        if [ -f "$db" ]; then
            title=$(sqlite3 "$db" "SELECT title FROM metadata LIMIT 1;" 2>/dev/null || echo "N/A")
            tasks=$(sqlite3 "$db" "SELECT COUNT(*) FROM tasks;" 2>/dev/null || echo "0")
            echo "  $name: $tasks tasks - $title"
        elif [ -d "${f}.ralph" ]; then
            echo "  $name: (no database)"
        fi
    done

# === Fixtures (Read-only reference data) ===

# List available fixtures (note: use external mock dir for testing)
list-fixtures:
    #!/usr/bin/env bash
    echo "Available fixtures (read-only, use 'just reset-mock' for testing):"
    for f in fixtures/*/; do
        name=$(basename "$f")
        db="${f}.undetect-ralph/db/ralph.db"
        if [ -f "$db" ]; then
            title=$(sqlite3 "$db" "SELECT title FROM metadata LIMIT 1;" 2>/dev/null || echo "N/A")
            tasks=$(sqlite3 "$db" "SELECT COUNT(*) FROM tasks;" 2>/dev/null || echo "0")
            echo "  $name: $tasks tasks - $title"
        elif [ -d "${f}.undetect-ralph" ]; then
            echo "  $name: (no database)"
        fi
    done
