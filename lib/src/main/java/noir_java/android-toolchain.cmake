set(CMAKE_SYSTEM_NAME Android)
set(CMAKE_SYSTEM_VERSION 33)
set(CMAKE_ANDROID_ARCH_ABI arm64-v8a)
set(CMAKE_ANDROID_NDK $ENV{ANDROID_NDK_HOME})
set(CMAKE_ANDROID_STL_TYPE c++_shared)

# Specify the cross compiler
set(CMAKE_C_COMPILER "${CMAKE_ANDROID_NDK}/toolchains/llvm/prebuilt/${HOST_TAG}/bin/aarch64-linux-android33-clang")
set(CMAKE_CXX_COMPILER "${CMAKE_ANDROID_NDK}/toolchains/llvm/prebuilt/${HOST_TAG}/bin/aarch64-linux-android33-clang++")

# Where is the target environment
set(CMAKE_FIND_ROOT_PATH "${CMAKE_ANDROID_NDK}/toolchains/llvm/prebuilt/${HOST_TAG}/sysroot")

# Search for programs only in the build host directories
set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)

# Search for libraries and headers only in the target directories
set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_PACKAGE ONLY)

# Disable macOS specific flags
set(CMAKE_OSX_DEPLOYMENT_TARGET "")
set(CMAKE_OSX_SYSROOT "")
set(CMAKE_XCODE_ATTRIBUTE_DEVELOPMENT_TEAM "")

# Compiler and linker flags
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -DANDROID -ffunction-sections -fdata-sections -fPIC")
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -DANDROID -ffunction-sections -fdata-sections -fPIC")
set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -Wl,--gc-sections") 