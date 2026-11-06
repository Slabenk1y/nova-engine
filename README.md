# NOVA 2.0 — Independent User-Generated Gaming Platform

Nova is an independent 3D mobile gaming platform and engine inspired by the feature set and user experience of Roblox, built in Rust for Android and future cross-platform deployment.

## Architecture & Subsystems
- **Game Runtime**: Unified runtime loading versioned 3D places, executing physics, characters, and scripts.
- **Physics**: Rigid bodies, character controller with jumping and ground detection, moving platforms, and vehicle dynamics.
- **Character & Avatar**: Block-style avatar rig, full color and slot customization, animation state machine, and inventory ownership validation.
- **Scripting**: Visual node/block scripting system and sandboxed text execution engine.
- **Multiplayer Network**: Authoritative server architecture, player synchronization, chat broadcasting, and room instances.
- **Backend & Cloud**: User accounts with SHA256 password hashing, profiles, friends, communities, catalog discovery, and persistent player storage.
- **NOVA Creator**: Studio model with scene hierarchy, undo/redo stack, starter templates, and playtest state preservation.
- **Android Client**: Native Android application packaging, activity lifecycle management, and touch virtual controls.

## 4 Mandatory Test Experiences
1. **NOVA Obby** (`experiences/nova_obby`): 3D obstacle course with floating steps, moving hazard platforms, lava floor triggers, and checkpoints.
2. **NOVA Social World** (`experiences/nova_social_world`): Multiplayer 3D central plaza with player synchronization and chat broadcasting.
3. **NOVA Simulator** (`experiences/nova_simulator`): Resource harvesting, backpack capacity, upgrade shop, and backend persistence.
4. **NOVA Racing** (`experiences/nova_racing`): Circuit Grand Prix with vehicle dynamics, track barriers, checkpoints, and lap timing.

## Building and Testing
```bash
# Run all workspace unit and integration tests
cargo test --workspace

# Build and run desktop launcher
cargo run -p nova-app

# Build signed Android APK
./android/build_apk.sh
```
The resulting verified signed APK is generated at `android/build/outputs/apk/nova-debug.apk`.
