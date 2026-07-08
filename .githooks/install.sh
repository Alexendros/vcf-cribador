#!/bin/sh
# Instala los hooks de git desde .githooks/ al directorio .git/hooks/

set -eu

HOOKS_DIR="$(cd "$(dirname "$0")" && pwd)"
GIT_DIR="$(git rev-parse --git-dir 2>/dev/null || echo '')"

if [ -z "$GIT_DIR" ]; then
    echo "No estás dentro de un repositorio git."
    exit 1
fi

echo "Instalando hooks desde $HOOKS_DIR hacia $GIT_DIR/hooks"

for hook in "$HOOKS_DIR"/*; do
    name="$(basename "$hook")"
    if [ "$name" = "install.sh" ]; then
        continue
    fi
    cp "$hook" "$GIT_DIR/hooks/$name"
    chmod +x "$GIT_DIR/hooks/$name"
    echo "  ✓ $name"
done

echo "Hooks instalados."
