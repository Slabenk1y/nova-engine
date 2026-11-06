#!/usr/bin/env bash
set -e

ANDROID_JAR="/opt/android-sdk/platforms/android-34/android.jar"
AAPT2="/opt/android-sdk/build-tools/34.0.0/aapt2"
D8="/opt/android-sdk/build-tools/34.0.0/d8"
ZIPALIGN="/opt/android-sdk/build-tools/34.0.0/zipalign"
APKSIGNER="/opt/android-sdk/build-tools/34.0.0/apksigner"

echo "=== 1. Building Native Rust Engine Libraries for Android ==="
cargo build -p nova-runtime --target aarch64-linux-android --release
cargo build -p nova-runtime --target x86_64-linux-android --release

echo "=== 2. Setting up APK build directory structure ==="
rm -rf android/build/compiled_res android/build/gen android/build/classes android/build/dex android/build/libs android/build/outputs/apk
mkdir -p android/build/compiled_res android/build/gen android/build/classes android/build/dex android/build/outputs/apk
mkdir -p android/build/libs/lib/arm64-v8a
mkdir -p android/build/libs/lib/x86_64

cp target/aarch64-linux-android/release/libnova_runtime.so android/build/libs/lib/arm64-v8a/libnova_engine.so
cp target/x86_64-linux-android/release/libnova_runtime.so android/build/libs/lib/x86_64/libnova_engine.so

echo "=== 3. Compiling Android Resources ==="
$AAPT2 compile --dir android/app/src/main/res -o android/build/compiled_res/

echo "=== 4. Linking Manifest and Resources ==="
$AAPT2 link -I "$ANDROID_JAR" \
    --manifest android/app/src/main/AndroidManifest.xml \
    -o android/build/nova-unaligned.apk \
    --java android/build/gen \
    android/build/compiled_res/*.flat \
    --auto-add-overlay

echo "=== 5. Compiling Java Source Files ==="
javac --release 17 -d android/build/classes \
    -cp "$ANDROID_JAR" \
    android/build/gen/com/nova/engine/R.java \
    android/app/src/main/java/com/nova/engine/NovaMainActivity.java

echo "=== 6. Compiling Dalvik Executable (classes.dex) ==="
$D8 --output android/build/dex/ \
    --lib "$ANDROID_JAR" \
    android/build/classes/com/nova/engine/*.class

echo "=== 7. Injecting classes.dex and native libraries (.so) ==="
cd android/build/dex
jar uf ../nova-unaligned.apk classes.dex
cd ../../..

cd android/build/libs
jar uf ../nova-unaligned.apk lib/arm64-v8a/libnova_engine.so lib/x86_64/libnova_engine.so
cd ../../..

echo "=== 8. Aligning APK ==="
$ZIPALIGN -f -p 4 android/build/nova-unaligned.apk android/build/outputs/apk/nova-debug.apk

echo "=== 9. Signing APK ==="
$APKSIGNER sign --ks android/keystore/debug.keystore --ks-pass pass:android --ks-key-alias androiddebugkey --key-pass pass:android android/build/outputs/apk/nova-debug.apk

echo "=== 10. Verifying APK Signature and Contents ==="
$APKSIGNER verify android/build/outputs/apk/nova-debug.apk
echo "APK successfully built, verified, and signed at: android/build/outputs/apk/nova-debug.apk"
