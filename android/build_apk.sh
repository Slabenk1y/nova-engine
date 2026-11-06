#!/usr/bin/env bash
set -e

ANDROID_JAR="/opt/android-sdk/platforms/android-34/android.jar"
AAPT2="/opt/android-sdk/build-tools/34.0.0/aapt2"
D8="/opt/android-sdk/build-tools/34.0.0/d8"
ZIPALIGN="/opt/android-sdk/build-tools/34.0.0/zipalign"
APKSIGNER="/opt/android-sdk/build-tools/34.0.0/apksigner"

echo "Building NOVA 2.0 Android APK..."

rm -rf android/build/compiled_res android/build/gen android/build/classes android/build/dex android/build/outputs/apk
mkdir -p android/build/compiled_res android/build/gen android/build/classes android/build/dex android/build/outputs/apk

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

$ZIPALIGN -f -p 4 android/build/nova-unaligned.apk android/build/outputs/apk/nova-debug.apk

$APKSIGNER sign --ks android/keystore/debug.keystore --ks-pass pass:android --ks-key-alias androiddebugkey --key-pass pass:android android/build/outputs/apk/nova-debug.apk

$APKSIGNER verify android/build/outputs/apk/nova-debug.apk

echo "Successfully built and verified: android/build/outputs/apk/nova-debug.apk"
