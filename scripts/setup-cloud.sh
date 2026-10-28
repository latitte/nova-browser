#!/usr/bin/env bash
set -euo pipefail
# Préparation locale, sans privilèges administrateur, pour Debian 13.
cd /workspace/nova-browser
mkdir -p /workspace/tooling/apt/lists/partial /workspace/tooling/apt/cache/archives/partial /workspace/tooling/sysroot
export CARGO_HOME=/workspace/tooling/cargo RUSTUP_HOME=/workspace/tooling/rustup
export npm_config_cache=/workspace/tooling/npm
if [ ! -x "$CARGO_HOME/bin/rustup" ]; then
  curl --fail --location https://sh.rustup.rs -o /workspace/tooling/rustup-init.sh
  sh /workspace/tooling/rustup-init.sh -y --profile minimal --default-toolchain 1.99.1 --no-modify-path
fi
printf 'deb https://deb.debian.org/debian trixie main\n' > /workspace/tooling/apt/sources.list
apt_options=(-o Debug::NoLocking=true -o Dir::Etc::sourcelist=/workspace/tooling/apt/sources.list -o Dir::Etc::sourceparts=- -o Dir::State=/workspace/tooling/apt -o Dir::State::status=/var/lib/dpkg/status -o Dir::Cache=/workspace/tooling/apt/cache -o APT::Sandbox::User=agent)
apt-get "${apt_options[@]}" update
apt-get "${apt_options[@]}" --download-only --no-install-recommends install -y libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev patchelf xvfb xauth dbus-x11
for package in /workspace/tooling/apt/cache/archives/*.deb; do dpkg-deb -x "$package" /workspace/tooling/sysroot; done
python3 - <<'PY'
from pathlib import Path
root = Path('/workspace/tooling/sysroot')
for p in root.rglob('*.pc'):
    p.write_text(p.read_text().replace('/usr', '/workspace/tooling/sysroot/usr'))
for p in root.rglob('*'):
    if p.is_symlink() and not p.exists():
        target = p.readlink()
        actual = p.parent / target if not target.is_absolute() else root / str(target).lstrip('/')
        try:
            system = Path('/') / actual.relative_to(root)
        except ValueError:
            continue
        if system.exists():
            p.unlink()
            p.symlink_to(system)
PY
cat > /workspace/tooling/activate-nova.sh <<'ACTIVATE'
export CARGO_HOME=/workspace/tooling/cargo
export RUSTUP_HOME=/workspace/tooling/rustup
export PATH=/workspace/tooling/cargo/bin:/workspace/tooling/sysroot/usr/bin:$PATH
export npm_config_cache=/workspace/tooling/npm
export PKG_CONFIG_PATH=/workspace/tooling/sysroot/usr/lib/x86_64-linux-gnu/pkgconfig:/workspace/tooling/sysroot/usr/share/pkgconfig
export LIBRARY_PATH=/workspace/tooling/sysroot/usr/lib/x86_64-linux-gnu
export LD_LIBRARY_PATH=/workspace/tooling/sysroot/usr/lib/x86_64-linux-gnu
export XDG_DATA_HOME=/workspace/tooling/user-data
export XDG_CACHE_HOME=/workspace/tooling/user-cache
export XDG_CONFIG_HOME=/workspace/tooling/user-config
export CARGO_BUILD_JOBS=4
ACTIVATE
. /workspace/tooling/activate-nova.sh
npm ci --no-audit --no-fund
npm run check
npm test
npm run build
node --test scripts/ui.test.mjs
cargo test --locked --manifest-path src-tauri/Cargo.toml
# L'extraction locale permet la compilation. Elle ne remplace pas une installation
# système de WebKitGTK pour l'exécution desktop : voir docs/VALIDATION.md.
