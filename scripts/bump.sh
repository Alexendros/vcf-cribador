#!/bin/sh
# scripts/bump.sh — Incrementa versión semántica y actualiza todos los archivos.
# Uso: ./scripts/bump.sh patch|minor|major

set -eu

CARGO_TOML="Cargo.toml"
README="README.md"
CHANGELOG="CHANGELOG.md"

# ── Leer versión actual ──
current=$(grep '^version' "$CARGO_TOML" | head -1 | sed 's/version = "\(.*\)"/\1/')

if [ -z "$current" ]; then
    echo "No se pudo leer la versión de $CARGO_TOML"
    exit 1
fi

# ── Parsear semver ──
major=$(echo "$current" | cut -d. -f1)
minor=$(echo "$current" | cut -d. -f2)
patch=$(echo "$current" | cut -d. -f3)

# ── Calcular nueva versión ──
case "${1:-}" in
    major)
        major=$((major + 1))
        minor=0
        patch=0
        ;;
    minor)
        minor=$((minor + 1))
        patch=0
        ;;
    patch)
        patch=$((patch + 1))
        ;;
    *)
        echo "Uso: $0 patch|minor|major"
        echo "Versión actual: $current"
        exit 1
        ;;
esac

new="$major.$minor.$patch"
today=$(date +%Y-%m-%d)

echo "→ $current → $new (${1})"

# ── Actualizar Cargo.toml ──
sed -i "s/^version = \"$current\"/version = \"$new\"/" "$CARGO_TOML"
echo "  ✓ $CARGO_TOML"

# ── Actualizar badge en README ──
sed -i "s/version-$current/version-$new/g" "$README" 2>/dev/null || true
echo "  ✓ $README (badge)"

# ── Actualizar CHANGELOG ──
if [ -f "$CHANGELOG" ]; then
    sed -i "s/^## \[Unreleased\]/## [Unreleased]\n\n## [$new] - $today/" "$CHANGELOG"
    echo "  ✓ $CHANGELOG"
fi

# ── Verificar con cargo check ──
echo "→ Verificando compilación..."
cargo check -q 2>&1 && echo "  ✓ cargo check OK"

echo ""
echo "Versión actualizada: $new"
echo ""
echo "Siguientes pasos:"
echo "  1. Revisa CHANGELOG.md y completa la sección [$new]"
echo "  2. git add -A && git commit -m \"chore: bump v$new\""
echo "  3. git tag v$new"
