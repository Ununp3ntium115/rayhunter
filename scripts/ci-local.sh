#!/bin/bash
# Run the checks from .github/workflows/main.yml (and release.yml's version
# check) locally. Useful when GitHub Actions is unavailable, and as a
# pre-push sanity check.
#
# Usage: ./scripts/ci-local.sh [--all] [--docs] [--rootshell] [--firmware] [--gui]
#
# Default jobs (no flags):
#   versions          release.yml check_version_same
#   check_and_test    cargo fmt --check, web build, cargo check/test/clippy
#   installer_nodef   windows_installer_check_and_test (cargo test --no-default-features)
#   frontend          test_daemon_frontend + test_installer_frontend
#
# Optional jobs:
#   --docs       mdbook_test + openapi_build (installs mdbook 0.4 via cargo if missing)
#   --rootshell  build_rootshell (armv7 musl, needs only rustup)
#   --firmware   build_rayhunter (armv7 daemon, needs docker)
#   --gui        installer_gui_check (cargo check/clippy -p installer-gui, needs tauri system deps)
#   --all        everything above
#
# Exits non-zero if any job fails, after running all selected jobs.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$(dirname "$SCRIPT_DIR")"

# Same as the workflow env: warnings fail the build.
export RUSTFLAGS="-Dwarnings"
export CARGO_TERM_COLOR=always

run_docs=0 run_rootshell=0 run_firmware=0 run_gui=0
for arg in "$@"; do
    case "$arg" in
        --docs) run_docs=1 ;;
        --rootshell) run_rootshell=1 ;;
        --firmware) run_firmware=1 ;;
        --gui) run_gui=1 ;;
        --all) run_docs=1 run_rootshell=1 run_firmware=1 run_gui=1 ;;
        -h|--help) sed -n '2,21p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "unknown argument: $arg" >&2; exit 2 ;;
    esac
done

passed=()
failed=()

job() {
    local name="$1"
    shift
    echo
    echo "=== $name"
    if "$@"; then
        passed+=("$name")
    else
        failed+=("$name")
        echo "=== $name FAILED"
    fi
}

versions() {
    local n
    n=$(find lib check daemon installer installer-gui rootshell telcom-parser -name Cargo.toml -exec grep ^version {} \; | sort -u | wc -l)
    if [ "$n" != "1" ]; then
        find lib check daemon installer installer-gui rootshell telcom-parser -name Cargo.toml -exec grep -H ^version {} \;
        echo "all Cargo.toml files must have the same version defined"
        return 1
    fi
}

check_and_test() {
    cargo fmt --all --check &&
        npm install --no-audit --no-fund &&
        npm run build -w daemon/web &&
        cargo check &&
        cargo test &&
        cargo clippy
}

installer_nodef() {
    (cd installer && cargo test --no-default-features)
}

frontend() {
    npm install --no-audit --no-fund &&
        npm run lint -w daemon/web &&
        npm run check -w daemon/web &&
        npm run test -w daemon/web &&
        npm run lint -w installer-gui &&
        npm run check -w installer-gui
}

docs() {
    if ! command -v mdbook > /dev/null; then
        cargo install mdbook --no-default-features --features search --vers "^0.4" --locked || return 1
    fi
    mdbook test || return 1
    # openapi_build: gen_api embeds the web build, like the daemon
    npm run build -w daemon/web &&
        cargo run --bin gen_api --features apidocs -- "${TMPDIR:-/tmp}/rayhunter-openapi.json"
}

rootshell() {
    rustup target add armv7-unknown-linux-musleabihf &&
        cargo build -p rootshell --bin rootshell --target armv7-unknown-linux-musleabihf --profile=firmware
}

firmware() {
    npm run build -w daemon/web || return 1
    mkdir -p "$HOME/.cargo-musl-cross" "$HOME/.rustup-musl-cross"
    docker run --rm \
        --user "$(id -u):$(id -g)" \
        -v "$PWD":/work \
        -v "$HOME/.cargo-musl-cross":/cargo-home \
        -v "$HOME/.rustup-musl-cross":/rustup-home \
        -e CARGO_HOME=/cargo-home \
        -e RUSTUP_HOME=/rustup-home \
        -e RUSTFLAGS \
        -w /work \
        messense/rust-musl-cross:armv7-musleabihf \
        sh -c 'rustup target add armv7-unknown-linux-musleabihf && cargo build-daemon-firmware'
}

gui() {
    cargo check --package installer-gui && cargo clippy --package installer-gui
}

job versions versions
job check_and_test check_and_test
job installer_nodef installer_nodef
job frontend frontend
[ "$run_docs" = 1 ] && job docs docs
[ "$run_rootshell" = 1 ] && job rootshell rootshell
[ "$run_firmware" = 1 ] && job firmware firmware
[ "$run_gui" = 1 ] && job gui gui

echo
echo "passed: ${passed[*]:-none}"
echo "failed: ${failed[*]:-none}"
[ "${#failed[@]}" -eq 0 ]
