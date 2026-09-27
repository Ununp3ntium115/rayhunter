#!/bin/bash
# Build the release artifacts that .github/workflows/main.yml produces, locally
# on a Linux x86_64 host. Use this when GitHub Actions is unavailable.
#
# Usage: ./scripts/release-local.sh [--no-gui] [--no-docs] [OUT_DIR]
#
# Produces in OUT_DIR (default: ./release-out):
#   release/rayhunter-vX.Y.Z-{linux-x64,linux-aarch64,linux-armv7,windows-x86_64}.zip{,.sha256}
#   gui/          Linux x64 GUI installer (AppImage, deb, rpm)          (skip: --no-gui)
#   pages/        mdBook site with api-docs/, as deployed to GitHub Pages (skip: --no-docs)
#
# macOS builds (rayhunter-check, installer, GUI) need Apple's SDK and are not
# produced; the same goes for the Windows GUI installer (MSVC).
#
# Host requirements (Debian/Ubuntu package names):
#   rustup, npm, docker, zip, gcc-arm-linux-gnueabihf, mingw-w64, flex, bison,
#   pkg-config, and for the GUI the Tauri deps listed in main.yml
#   (libwebkit2gtk-4.1-dev libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev).
# If HTTPS_PROXY/SSL_CERT_FILE are set they are passed into the firmware
# build container.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$(dirname "$SCRIPT_DIR")"

build_gui=1 build_docs=1 out=""
for arg in "$@"; do
    case "$arg" in
        --no-gui) build_gui=0 ;;
        --no-docs) build_docs=0 ;;
        -h|--help) sed -n '2,20p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        -*) echo "unknown argument: $arg" >&2; exit 2 ;;
        *) out="$arg" ;;
    esac
done
out="$(mkdir -p "${out:-release-out}" && cd "${out:-release-out}" && pwd)"

export RUSTFLAGS="-Dwarnings"
# installer/build.rs embeds firmware from ./target, so use the default target dir.
unset CARGO_TARGET_DIR

declare -A TARGETS=(
    [linux-x64]=x86_64-unknown-linux-musl
    [linux-armv7]=armv7-unknown-linux-musleabi
    [linux-aarch64]=aarch64-unknown-linux-musl
    [windows-x86_64]=x86_64-pc-windows-gnu
)
FW=target/armv7-unknown-linux-musleabihf/firmware

step() { echo; echo "=== $*"; }

step "frontend"
npm install --no-audit --no-fund
npm run build -w daemon/web

step "build_rootshell"
rustup target add armv7-unknown-linux-musleabihf
cargo build -p rootshell --bin rootshell --target armv7-unknown-linux-musleabihf --profile=firmware

step "build_rayhunter (daemon firmware, in messense/rust-musl-cross like CI)"
mkdir -p "$HOME/.cargo-musl-cross" "$HOME/.rustup-musl-cross"
docker_args=(--rm --user "$(id -u):$(id -g)"
    -v "$PWD":/work -w /work
    -v "$HOME/.cargo-musl-cross":/cargo-home -v "$HOME/.rustup-musl-cross":/rustup-home
    -e CARGO_HOME=/cargo-home -e RUSTUP_HOME=/rustup-home -e RUSTFLAGS)
if [ -n "${HTTPS_PROXY:-}" ]; then
    # the proxy may listen on the host's loopback
    docker_args+=(--network host -e HTTPS_PROXY -e HTTP_PROXY -e "https_proxy=$HTTPS_PROXY")
fi
if [ -n "${SSL_CERT_FILE:-}" ]; then
    docker_args+=(-v "$SSL_CERT_FILE":/ca.crt:ro -e SSL_CERT_FILE=/ca.crt -e CARGO_HTTP_CAINFO=/ca.crt)
fi
docker run "${docker_args[@]}" messense/rust-musl-cross:armv7-musleabihf \
    sh -c 'rustup target add armv7-unknown-linux-musleabihf && cargo build-daemon-firmware'

step "build_wpa_supplicant"
CC=arm-linux-gnueabihf-gcc STRIP=arm-linux-gnueabihf-strip HOST=arm-linux-gnueabihf \
    scripts/build-wpa-supplicant.sh

step "build_rayhunter_check + build_rust_installer"
for platform in "${!TARGETS[@]}"; do
    target="${TARGETS[$platform]}"
    rustup target add "$target"
    cargo build --bin rayhunter-check --release --target "$target"
    cargo build --package installer --bin installer --release --target "$target"
done

step "build_release_zip"
version=$(grep '^version' daemon/Cargo.toml | head -n 1 | cut -d'"' -f2)
stage="$(mktemp -d)"
trap 'rm -rf "$stage"' EXIT
mkdir -p "$stage/rayhunter-daemon"
cp "$FW/rayhunter-daemon" "$stage/rayhunter-daemon/"
for platform in "${!TARGETS[@]}"; do
    ext=""
    [ "$platform" = windows-x86_64 ] && ext=".exe"
    mkdir -p "$stage/rayhunter-check-$platform"
    cp "target/${TARGETS[$platform]}/release/rayhunter-check$ext" "$stage/rayhunter-check-$platform/"
done
mkdir -p "$out/release"
for platform in "${!TARGETS[@]}"; do
    ext=""
    [ "$platform" = windows-x86_64 ] && ext=".exe"
    dest="rayhunter-v${version}-${platform}"
    rm -rf "${stage:?}/$dest"
    mkdir "$stage/$dest"
    cp "target/${TARGETS[$platform]}/release/installer$ext" "$stage/$dest/"
    cp -r "$stage"/rayhunter-check-* "$stage/rayhunter-daemon" dist/scripts "$stage/$dest/"
    chmod +x "$stage/$dest"/installer* "$stage/$dest"/rayhunter-check-*/rayhunter-check* "$stage/$dest/rayhunter-daemon/rayhunter-daemon"
    rm -f "$out/release/$dest.zip"
    (cd "$stage" && zip -qr "$out/release/$dest.zip" "$dest")
    (cd "$out/release" && sha256sum "$dest.zip" > "$dest.zip.sha256")
done

if [ "$build_gui" = 1 ]; then
    step "build_installer_gui_linux (x64)"
    npm run tauri build -w installer-gui -- --target x86_64-unknown-linux-gnu
    mkdir -p "$out/gui"
    cp target/x86_64-unknown-linux-gnu/release/bundle/appimage/*.AppImage \
        target/x86_64-unknown-linux-gnu/release/bundle/deb/*.deb \
        target/x86_64-unknown-linux-gnu/release/bundle/rpm/*.rpm "$out/gui/"
fi

if [ "$build_docs" = 1 ]; then
    step "mdbook_build + openapi_build"
    command -v mdbook > /dev/null ||
        cargo install mdbook --no-default-features --features search --vers "^0.4" --locked
    rm -rf "$out/pages"
    mdbook build -d "$out/pages"
    mkdir -p "$out/pages/api-docs"
    cargo run --bin gen_api --features apidocs -- "$out/pages/api-docs/rayhunter-openapi.json"
    cp doc/swagger-ui.html "$out/pages/api-docs/index.html"
fi

step "done"
find "$out" -maxdepth 2 -type f \( -name '*.zip' -o -name '*.AppImage' -o -name '*.deb' -o -name '*.rpm' \) -exec ls -la {} \;
