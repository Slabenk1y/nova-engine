#!/usr/bin/env bash
set -e

echo "=== NOVA 2.0 Android Build Script ==="

# 1. Resolve Android SDK Path
if [ -n "$ANDROID_HOME" ] && [ -d "$ANDROID_HOME" ]; then
    SDK_DIR="$ANDROID_HOME"
elif [ -n "$ANDROID_SDK_ROOT" ] && [ -d "$ANDROID_SDK_ROOT" ]; then
    SDK_DIR="$ANDROID_SDK_ROOT"
elif [ -d "/opt/android-sdk" ]; then
    SDK_DIR="/opt/android-sdk"
elif [ -d "/usr/local/lib/android/sdk" ]; then
    SDK_DIR="/usr/local/lib/android/sdk"
elif [ -d "$HOME/Android/Sdk" ]; then
    SDK_DIR="$HOME/Android/Sdk"
else
    echo "ERROR: Android SDK not found. Set ANDROID_HOME or ANDROID_SDK_ROOT."
    exit 1
fi
echo "Using Android SDK: $SDK_DIR"

# 2. Resolve Android NDK Path
if [ -n "$ANDROID_NDK_HOME" ] && [ -d "$ANDROID_NDK_HOME" ]; then
    NDK_DIR="$ANDROID_NDK_HOME"
elif [ -n "$ANDROID_NDK_ROOT" ] && [ -d "$ANDROID_NDK_ROOT" ]; then
    NDK_DIR="$ANDROID_NDK_ROOT"
elif [ -d "$SDK_DIR/ndk" ]; then
    NDK_DIR=$(find "$SDK_DIR/ndk" -mindepth 1 -maxdepth 1 -type d | sort -V | tail -n 1)
elif [ -d "$SDK_DIR/ndk-bundle" ]; then
    NDK_DIR="$SDK_DIR/ndk-bundle"
fi

if [ -z "$NDK_DIR" ] || [ ! -d "$NDK_DIR" ]; then
    echo "ERROR: Android NDK not found in $SDK_DIR/ndk."
    exit 1
fi
echo "Using Android NDK: $NDK_DIR"

# 3. Resolve Toolchain Linkers
NDK_BIN_DIR=$(find "$NDK_DIR/toolchains/llvm/prebuilt" -mindepth 1 -maxdepth 2 -name "bin" -type d | head -n 1)
if [ -z "$NDK_BIN_DIR" ] || [ ! -d "$NDK_BIN_DIR" ]; then
    echo "ERROR: LLVM prebuilt toolchain bin not found in $NDK_DIR."
    exit 1
fi

ARM64_CLANG=$(find "$NDK_BIN_DIR" -name "aarch64-linux-android*-clang" | grep -v "++" | sort -V | head -n 1)
X86_64_CLANG=$(find "$NDK_BIN_DIR" -name "x86_64-linux-android*-clang" | grep -v "++" | sort -V | head -n 1)

if [ -z "$ARM64_CLANG" ] || [ -z "$X86_64_CLANG" ]; then
    echo "ERROR: Could not find target clang compilers in $NDK_BIN_DIR."
    exit 1
fi
echo "ARM64 Clang: $ARM64_CLANG"
echo "x86_64 Clang: $X86_64_CLANG"

export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$ARM64_CLANG"
export CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="$X86_64_CLANG"

# 4. Resolve Build Tools and Platforms
BUILD_TOOLS_DIR=$(find "$SDK_DIR/build-tools" -mindepth 1 -maxdepth 1 -type d | sort -V | tail -n 1)
if [ -z "$BUILD_TOOLS_DIR" ] || [ ! -d "$BUILD_TOOLS_DIR" ]; then
    echo "ERROR: Android build-tools not found in $SDK_DIR/build-tools."
    exit 1
fi
echo "Using Build Tools: $BUILD_TOOLS_DIR"

AAPT2="$BUILD_TOOLS_DIR/aapt2"
D8="$BUILD_TOOLS_DIR/d8"
ZIPALIGN="$BUILD_TOOLS_DIR/zipalign"
APKSIGNER="$BUILD_TOOLS_DIR/apksigner"

for tool in "$AAPT2" "$D8" "$ZIPALIGN" "$APKSIGNER"; do
    if [ ! -f "$tool" ]; then
        echo "ERROR: Required tool missing: $tool"
        exit 1
    fi
done

PLATFORM_DIR=$(find "$SDK_DIR/platforms" -mindepth 1 -maxdepth 1 -name "android-*" -type d | sort -V | tail -n 1)
if [ -z "$PLATFORM_DIR" ] || [ ! -f "$PLATFORM_DIR/android.jar" ]; then
    echo "ERROR: android.jar not found in $SDK_DIR/platforms."
    exit 1
fi
ANDROID_JAR="$PLATFORM_DIR/android.jar"
echo "Using Android Platform: $ANDROID_JAR"

# 5. Build Native Rust Libraries
echo "=== Building Native Rust Engine Libraries ==="
cargo build -p nova-runtime --target aarch64-linux-android --release
cargo build -p nova-runtime --target x86_64-linux-android --release

# 6. Assembly APK
echo "=== Assembling Android APK ==="
rm -rf android/build/compiled_res android/build/gen android/build/classes android/build/dex android/build/libs android/build/outputs/apk
mkdir -p android/build/compiled_res android/build/gen android/build/classes android/build/dex android/build/outputs/apk
mkdir -p android/build/libs/lib/arm64-v8a
mkdir -p android/build/libs/lib/x86_64

cp target/aarch64-linux-android/release/libnova_runtime.so android/build/libs/lib/arm64-v8a/libnova_engine.so
cp target/x86_64-linux-android/release/libnova_runtime.so android/build/libs/lib/x86_64/libnova_engine.so

$AAPT2 compile --dir android/app/src/main/res -o android/build/compiled_res/

$AAPT2 link -I "$ANDROID_JAR" \
    --manifest android/app/src/main/AndroidManifest.xml \
    -o android/build/nova-unaligned.apk \
    --java android/build/gen \
    android/build/compiled_res/*.flat \
    --auto-add-overlay

javac --release 17 -d android/build/classes \
    -cp "$ANDROID_JAR" \
    android/build/gen/com/nova/engine/R.java \
    android/app/src/main/java/com/nova/engine/NovaMainActivity.java

$D8 --output android/build/dex/ \
    --lib "$ANDROID_JAR" \
    android/build/classes/com/nova/engine/*.class

cd android/build/dex
jar uf ../nova-unaligned.apk classes.dex
cd ../../..

cd android/build/libs
jar uf ../nova-unaligned.apk lib/arm64-v8a/libnova_engine.so lib/x86_64/libnova_engine.so
cd ../../..

$ZIPALIGN -f -p 4 android/build/nova-unaligned.apk android/build/outputs/apk/nova-debug.apk

mkdir -p android/keystore
if [ ! -f android/keystore/debug.keystore ]; then
    echo "Generating debug keystore..."
    keytool -genkeypair -v \
        -keystore android/keystore/debug.keystore \
        -storepass android \
        -alias androiddebugkey \
        -keypass android \
        -keyalg RSA \
        -keysize 2048 \
        -validity 10000 \
        -dname "CN=Android Debug,O=Android,C=US"
fi

$APKSIGNER sign --ks android/keystore/debug.keystore --ks-pass pass:android --ks-key-alias androiddebugkey --key-pass pass:android android/build/outputs/apk/nova-debug.apk

$APKSIGNER verify --verbose android/build/outputs/apk/nova-debug.apk

echo "APK successfully built, verified, and signed at: android/build/outputs/apk/nova-debug.apk"
