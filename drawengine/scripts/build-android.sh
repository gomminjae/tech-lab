#!/usr/bin/env bash
set -euo pipefail

# Android build script using cargo-ndk
# Prerequisites: cargo install cargo-ndk, Android NDK installed

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="${1:-$PROJECT_DIR/target/android}"

TARGETS=(
    "aarch64-linux-android"
    "armv7-linux-androideabi"
    "x86_64-linux-android"
)

echo "=== Building drawengine-ffi for Android ==="

for target in "${TARGETS[@]}"; do
    echo "--- Building for $target ---"
    cargo ndk --target "$target" --platform 21 -- build --release -p drawengine-ffi
done

echo "=== Copying .so files ==="

mkdir -p "$OUTPUT_DIR/jniLibs/arm64-v8a"
mkdir -p "$OUTPUT_DIR/jniLibs/armeabi-v7a"
mkdir -p "$OUTPUT_DIR/jniLibs/x86_64"

cp "$PROJECT_DIR/target/aarch64-linux-android/release/libdrawengine_ffi.so" \
   "$OUTPUT_DIR/jniLibs/arm64-v8a/"

cp "$PROJECT_DIR/target/armv7-linux-androideabi/release/libdrawengine_ffi.so" \
   "$OUTPUT_DIR/jniLibs/armeabi-v7a/"

cp "$PROJECT_DIR/target/x86_64-linux-android/release/libdrawengine_ffi.so" \
   "$OUTPUT_DIR/jniLibs/x86_64/"

echo "=== Generating Kotlin bindings ==="
mkdir -p "$OUTPUT_DIR/kotlin"
cargo run -p uniffi-bindgen -- generate \
    --library "$PROJECT_DIR/target/aarch64-linux-android/release/libdrawengine_ffi.so" \
    --language kotlin \
    --out-dir "$OUTPUT_DIR/kotlin"

echo "=== Android build complete ==="
echo "JNI libs: $OUTPUT_DIR/jniLibs/"
echo "Kotlin bindings: $OUTPUT_DIR/kotlin/"
