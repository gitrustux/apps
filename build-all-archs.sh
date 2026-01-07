#!/bin/bash
set -e

ARCHITECTURES=(
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu"
    "riscv64gc-unknown-linux-gnu"
)

PACKAGES=(
    "rpg"
    "pkg-compat"
    "svc"
    "ip"
    "login"
    "ping"
    "fwctl"
    "tar"
    "rustux-dnslookup"
    "rustux-editor"
    "rustux-ssh"
    "rustux-logview"
    "rustux-apt"
    "rustux-apt-get"
    "rustux-capctl"
    "rustux-sbctl"
    "rustux-bootctl"
)

echo "======================================"
echo "Rustica Cross-Compilation Build"
echo "======================================"
echo ""

for arch in "${ARCHITECTURES[@]}"; do
    echo "Building for $arch..."
    for pkg in "${PACKAGES[@]}"; do
        cargo build -p "$pkg" --release --target "$arch" 2>&1 | grep -E "(Compiling|Finished|error)" || true
    done
    echo "Completed $arch"
    echo ""
done

echo "======================================"
echo "Cross-compilation complete!"
echo "======================================"
