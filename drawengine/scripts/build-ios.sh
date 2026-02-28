#!/usr/bin/env bash
set -euo pipefail

# iOS build script for multi-arch universal binary + xcframework
# Prerequisites: Xcode, rustup targets installed

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="${1:-$PROJECT_DIR/target/ios}"

TARGETS=(
    "aarch64-apple-ios"
    "aarch64-apple-ios-sim"
)

echo "=== Building drawengine-ffi for iOS ==="

for target in "${TARGETS[@]}"; do
    echo "--- Building for $target ---"
    cargo build --release --target "$target" -p drawengine-ffi
done

echo "=== Creating xcframework ==="

FRAMEWORK_NAME="DrawEngineFFI"
HEADERS_DIR="$OUTPUT_DIR/headers"
mkdir -p "$HEADERS_DIR"

# Generate Swift bindings and header
cargo run -p uniffi-bindgen -- generate \
    --library "$PROJECT_DIR/target/aarch64-apple-ios/release/libdrawengine_ffi.a" \
    --language swift \
    --out-dir "$OUTPUT_DIR/swift"

# Copy the generated header
if [ -f "$OUTPUT_DIR/swift/${FRAMEWORK_NAME}FFI.h" ]; then
    cp "$OUTPUT_DIR/swift/${FRAMEWORK_NAME}FFI.h" "$HEADERS_DIR/"
fi

# Create modulemap
cat > "$HEADERS_DIR/module.modulemap" << 'MODULEMAP'
module DrawEngineFFIFFI {
    header "DrawEngineFFIFFI.h"
    export *
}
MODULEMAP

# Create xcframework
rm -rf "$OUTPUT_DIR/$FRAMEWORK_NAME.xcframework"

xcodebuild -create-xcframework \
    -library "$PROJECT_DIR/target/aarch64-apple-ios/release/libdrawengine_ffi.a" \
    -headers "$HEADERS_DIR" \
    -library "$PROJECT_DIR/target/aarch64-apple-ios-sim/release/libdrawengine_ffi.a" \
    -headers "$HEADERS_DIR" \
    -output "$OUTPUT_DIR/$FRAMEWORK_NAME.xcframework"

echo "=== iOS build complete ==="
echo "xcframework: $OUTPUT_DIR/$FRAMEWORK_NAME.xcframework"
echo "Swift bindings: $OUTPUT_DIR/swift/"
