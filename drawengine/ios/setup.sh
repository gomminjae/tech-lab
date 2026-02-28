#!/usr/bin/env bash
set -euo pipefail

# Build xcframework and copy Swift bindings into the DrawExample project.
# Run from: drawengine/ios/

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=== Step 1: Build iOS xcframework ==="
"$PROJECT_DIR/scripts/build-ios.sh"

echo ""
echo "=== Step 2: Copy Swift bindings ==="
GENERATED_DIR="$SCRIPT_DIR/DrawExample/DrawExample/Generated"
mkdir -p "$GENERATED_DIR"

BINDINGS_SRC="$PROJECT_DIR/target/ios/swift"
if [ -f "$BINDINGS_SRC/DrawEngineFFI.swift" ]; then
    cp "$BINDINGS_SRC/DrawEngineFFI.swift" "$GENERATED_DIR/"
    echo "Copied DrawEngineFFI.swift → $GENERATED_DIR/"
else
    echo "ERROR: DrawEngineFFI.swift not found at $BINDINGS_SRC"
    echo "Make sure build-ios.sh completed successfully."
    exit 1
fi

echo ""
echo "=== Done ==="
echo ""
echo "Next steps in Xcode:"
echo "  1. Open DrawExample.xcodeproj"
echo "  2. Drag target/ios/DrawEngineFFI.xcframework into Frameworks"
echo "  3. Build & Run on simulator"
