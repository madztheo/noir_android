#!/bin/bash
set -e

# Set the NDK path
export NDK_VERSION="26.3.11579264"
export ANDROID_NDK_HOME="$HOME/Library/Android/sdk/ndk/$NDK_VERSION"
export HOST_TAG="darwin-x86_64"

# Add NDK tools to PATH
export PATH="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin:$PATH"

# Set up custom CMake for bb_rs
export CMAKE_TOOLCHAIN_FILE=$(pwd)/android-toolchain.cmake

# Verify toolchain file exists
if [ -f "$CMAKE_TOOLCHAIN_FILE" ]; then
    echo "Using CMake toolchain file: $CMAKE_TOOLCHAIN_FILE"
else
    echo "ERROR: CMake toolchain file not found: $CMAKE_TOOLCHAIN_FILE"
    exit 1
fi

# Clean up any previous build artifacts
echo "Cleaning previous build artifacts..."
rm -rf target/aarch64-linux-android

# Build the project
echo "Building for aarch64-linux-android with custom toolchain..."
RUST_BACKTRACE=1 cargo build --target aarch64-linux-android 