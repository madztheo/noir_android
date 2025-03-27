#!/bin/bash

# Set the NDK path - modify this according to your NDK version
export ANDROID_NDK_HOME="$HOME/Library/Android/sdk/ndk/26.3.11579264"

# Set NDK version for bb_rs build script
export NDK_VERSION="26.3.11579264"

# Set HOST_TAG for the bb_rs build script
export HOST_TAG="darwin-x86_64"

# Add NDK tools to PATH
export PATH="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin:$PATH"

# Create symbolic links for clang/clang++ with Android API level 33
cd "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin"
for arch in aarch64 armv7a i686 x86_64; do
    target="${arch}-linux-android"
    if [ "$arch" = "armv7a" ]; then
        target="arm-linux-androideabi"
    fi
    
    # Create symlinks for API level 33 if they don't exist
    if [ ! -f "${arch}-linux-android33-clang" ]; then
        ln -sf "${arch}-linux-android33-clang++" "${arch}-linux-android-clang++"
        ln -sf "${arch}-linux-android33-clang" "${arch}-linux-android-clang"
    fi
done
cd -

# Verify that the compiler is available
if command -v aarch64-linux-android33-clang >/dev/null 2>&1; then
    echo "Android NDK compiler found: $(which aarch64-linux-android33-clang)"
else
    echo "Warning: Android NDK compiler not found in PATH"
    echo "PATH=$PATH"
fi

# Set up Rust to target Android
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android

# Note: We're targeting Android API level 33 in our .cargo/config.toml

echo "Android NDK environment set up. Use 'cargo build --target aarch64-linux-android' to build for Android API level 33."
echo "NDK_VERSION=${NDK_VERSION}"
echo "HOST_TAG=${HOST_TAG}" 