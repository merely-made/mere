#!/bin/sh
# The engine-free configuration of the graph canvas: mere-cartography `canvas` with
# `physics-engine` off, and the crates that forward the feature, must compile,
# and rapier, parry and nalgebra must not be in the canvas's normal dependency
# tree. The viewer's main module is built this way (Graphshell host plan, T7).
# Run from the repository root.
set -eu

cargo check --locked -p mere-cartography --no-default-features --features canvas
cargo check --locked -p graphshell-host --no-default-features --features board

for pkg in mere-cartography graphshell-host; do
    case $pkg in
        mere-cartography) features=canvas ;;
        graphshell-host) features=board ;;
    esac
    tree=$(cargo tree --locked -p "$pkg" --no-default-features --features "$features" -e normal --prefix none)
    if printf '%s\n' "$tree" | grep -E '^(rapier2d|rapier|parry2d|parry|nalgebra|simba|glamx) '; then
        echo "$pkg with physics-engine off still links the engine" >&2
        exit 1
    fi
done
echo "physics-engine off: mere-cartography and graphshell-host build, with no rapier, parry or nalgebra in their normal trees"
