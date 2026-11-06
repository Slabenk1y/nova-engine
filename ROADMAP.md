# NOVA Roadmap

## Phase 0: Inspection & Foundation [COMPLETED]
- Cargo workspace with modular crates (`nova-core`, `nova-math`, `nova-physics`).
- Android SDK toolchain verification.

## Phase 1: Engine Core & Architecture [COMPLETED]
- Scene graph, versioned JSON serialization (`nova-world`).
- Character controller, avatar customization, animation state machine (`nova-character`).
- Visual node graph scripting engine (`nova-scripting`).

## Phase 2: Runtime, UI & Android Packaging [COMPLETED]
- Unified execution runtime (`nova-runtime`).
- 9 navigation sections & virtual touch controls (`nova-ui`).
- Android packaging script producing verified signed APK (`android/build/outputs/apk/nova-debug.apk`).

## Phase 3: Mandatory Test Experiences [COMPLETED]
- NOVA Obby: Obstacle course with moving hazards and checkpoints.
- NOVA Social World: Multiplayer social hub with chat and synchronization.
- NOVA Simulator: Progression miner with backend persistence.
- NOVA Racing: Vehicle circuit with lap tracking and checkpoint timing.

## Phase 4: Creator Studio & Publishing [COMPLETED]
- Undo/redo stack, starter templates, playtest state preservation (`nova-creator`).
- Backend publishing, moderation reporting, and catalog indexing (`nova-backend`).

## Phase 5: Production Hardening [IN PROGRESS / NEXT]
- WGPU / Vulkan native rendering surface integration on Android.
- Real-time UDP network transport layer (e.g. Quinn / WebRTC).
- Expanded asset importing for GLTF/OBJ 3D meshes.
