# NOVA Test Report

## Test Execution Details
Ran `cargo test --workspace` on all workspace members:
- `nova-core`: 2 passed (id uniqueness, engine clock)
- `nova-math`: 3 passed (vector operations, aabb intersection, ray intersection)
- `nova-physics`: 2 passed (gravity & grounding, vehicle acceleration)
- `nova-world`: 1 passed (place creation & versioned serialization)
- `nova-character`: 2 passed (animation state transitions, inventory ownership)
- `nova-scripting`: 1 passed (visual graph execution)
- `nova-network`: 1 passed (multiplayer server room & snapshot replication)
- `nova-backend`: 1 passed (authentication & persistent player data)
- `nova-ui`: 1 passed (virtual joystick mechanics)
- `nova-runtime`: 1 passed (runtime session loop & fall-respawn)
- `nova-creator`: 2 passed (undo/redo actions, playtest state preservation)
- `nova-app`: 1 passed (app launch and lifecycle)
- `experience-nova-obby`: 1 passed (obstacle course progression & finish)
- `experience-nova-social-world`: 1 passed (two-player chat & room synchronization)
- `experience-nova-simulator`: 1 passed (resource harvesting, backpack limits, upgrades, backend persistence)
- `experience-nova-racing`: 1 passed (circuit checkpoints, lap timing, finish line)

**Total Workspace Test Status**: 21 tests run, 21 passed, 0 failed across 16 packages.

## Android Build Verification
- **APK Target**: `android/build/outputs/apk/nova-debug.apk` (461 KB)
- **SHA-256 Checksum**: `277d4db287803b82b709774b30cc8e85657b6549872fd18b3602875249aff360`
- **AAPT Badging**: Package `com.nova.engine`, version 1.0.0, SDK min 24, target 34, landscape orientation.
- **Native Shared Libraries**: Bundled `lib/arm64-v8a/libnova_engine.so` and `lib/x86_64/libnova_engine.so` compiled via NDK r26d clang toolchain using official `jni` (v0.21.1) crate bindings (`env.get_direct_buffer_address(&out_direct_buffer)`).
- **Keystore CI Portability**: `android/build_apk.sh` checks for existence of `android/keystore/debug.keystore` and automatically generates it via `keytool` if absent.
- **DEX Compilation**: D8 generated `classes.dex` integrating `NovaMainActivity` with `GLSurfaceView.Renderer` and `TouchOverlay`.
- **APK Signature**: Verified with `apksigner verify --verbose`. Result: Validated using APK Signature Scheme v2 & v3.
- **Release Automation**: `.github/workflows/android-build-release.yml` configured to build with portable SDK/NDK discovery and upload the APK on every push as a GitHub Actions artifact (`nova-obby-android-debug-apk`) and GitHub Release on version tags.
- **CI Failure Diagnosis & Fix**: Diagnosed failure in GitHub Actions run 42296282817 (`android-actions/setup-android@v3` failed trying to download deprecated `tools` package). Fixed by removing the broken action and using `sdkmanager` directly with license acceptance. Fixed `android/build_apk.sh` to dynamically detect SDK, NDK, toolchain linkers, and build-tools without hardcoded paths.
- **Device & Emulator Testing Status**: As verified via `adb devices`, no physical Android phone or Android Virtual Device (AVD) emulator is available in this headless execution environment. Full packaging, JNI symbol exports (`nm -D`), manifest syntax, ABI library presence, and cryptographic signatures were verified programmatically using Android SDK build tools.
