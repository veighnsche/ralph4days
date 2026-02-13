# Ralph Loop development commands
# Run with: just <command>
# Install just: cargo install just

set shell := ["bash", "-cu"]
mock_dir := env_var_or_default('RALPH_MOCK_DIR', '/tmp/ralph4days-mock')

# Generate a discipline portrait: just gen-image 02 00 [--test|--half] [--ratio W H|--ratio-portrait] [--mp N]
gen-image STACK DISCIPLINE *FLAGS:
    cargo run -p predefined-disciplines --bin generate-discipline-image -- {{STACK}} {{DISCIPLINE}} {{FLAGS}}

# Default recipe: show available commands
default:
    @just --list

# === Development ===

# Start development server (frontend + backend hot reload)
dev:
    WEBKIT_DISABLE_DMABUF_RENDERER=1 bun tauri dev

# Start frontend dev server only
dev-frontend:
    bun dev

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

    WEBKIT_DISABLE_DMABUF_RENDERER=1 bun tauri dev -- -- --project "$PROJECT_DIR"

# Run cargo check (fast compilation check)
check:
    cargo check --manifest-path src-tauri/Cargo.toml

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

# === Testing ===

# Run all tests
test: test-rust test-frontend

# Run Rust tests
test-rust:
    cargo test --manifest-path src-tauri/Cargo.toml

# Run backend terminal-bridge test suite only
test-terminal-bridge-backend:
    cargo test --manifest-path src-tauri/Cargo.toml terminal_bridge
    cargo test --manifest-path src-tauri/Cargo.toml terminal::manager::tests

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

# Verify active e2e runtime surface has no forbidden browser-e2e framework references
audit-no-playwright:
    bun run audit:no-playwright

# === Building ===

# Build release binary (optimized for Alder Lake)
build:
    bun tauri build

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
    bun tauri build --bundles deb,rpm,appimage

# === Utilities ===

# Temporary: best-effort model discovery for Codex CLI
get-codex-models:
    cargo run --manifest-path src-tauri/Cargo.toml --bin provider-models -- codex

# Temporary: best-effort model discovery for Claude CLI
get-claude-models:
    cargo run --manifest-path src-tauri/Cargo.toml --bin provider-models -- claude

# Check if mold linker is installed
check-mold:
    @which mold > /dev/null && echo "✓ mold linker installed" || echo "✗ mold not found - install with: sudo dnf install mold"

# Show system info relevant to development
sysinfo:
    @echo "=== CPU ==="
    @lscpu | grep "Model name"
    @echo "\n=== Memory ==="
    @free -h | head -2
    @echo "\n=== GPU ==="
    @nvidia-smi --query-gpu=name,memory.total --format=csv,noheader 2>/dev/null || echo "No NVIDIA GPU"
    @echo "\n=== Rust ==="
    @rustc --version
    @echo "\n=== Node ==="
    @node --version

# Open project in VS Code
code:
    code .

# Watch for file changes and run tests
watch-test:
    cargo watch --manifest-path src-tauri/Cargo.toml -x test

# Generate TypeScript types from Rust via ts-rs (single file, no barrel)
types:
    #!/usr/bin/env bash
    rm -rf target/ts-bindings
    cargo test --workspace -- export_bindings 2>/dev/null || true
    echo '// Auto-generated by ts-rs — do not edit. Regenerate: just types' > src/types/generated.ts
    grep -hvE '^(import type|//)' target/ts-bindings/*.ts | sed '/^\/\*\*/,/\*\//d' | grep -v '^$' >> src/types/generated.ts
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
