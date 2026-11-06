# NOVA Architecture Documentation

## Overview
NOVA 2.0 is partitioned into modular, highly decoupled crates:

```
crates/
  ├── nova-core        # Primitives: Id, EngineTime, EngineError
  ├── nova-math        # 3D Math: Vec2, Vec3, Quat, Transform, AABB, Ray
  ├── nova-physics     # Physics engine, rigid bodies, character controller, vehicles
  ├── nova-world       # Places, multi-place experience manifests, entities, versioned JSON
  ├── nova-character   # Block avatar rig, customization, animation state machine, inventory
  ├── nova-scripting   # Visual node graphs and sandboxed execution runtime
  ├── nova-network     # Authoritative server rooms, packets, snapshot replication
  ├── nova-backend     # Accounts, authentication, catalog search, friends, data store
  ├── nova-creator     # Studio model, undo/redo stack, starter templates, playtesting
  ├── nova-ui          # 9 primary navigation tabs, touch joystick & buttons
  ├── nova-runtime     # Unified execution loop linking all systems together
  └── nova-app         # Desktop launcher and engine entry point

experiences/
  ├── nova_obby          # Playable Obstacle Course
  ├── nova_social_world  # Playable Multiplayer Social Hub
  ├── nova_simulator     # Playable Crystal Miner with Persistence
  └── nova_racing        # Playable Circuit Racing Experience

android/
  ├── app/               # Android Manifest, Java lifecycle activity, resources
  ├── keystore/          # Debug signing keystore
  └── build_apk.sh       # Script building signed APK via Android SDK build-tools
```

## Data Flow
1. **Catalog & Launch**: User navigates the 9 main tabs via `nova-ui`, selects an experience from `nova-backend`, and launches a `Place`.
2. **Runtime Execution**: `nova-runtime` instantiates a `RuntimeSession`, spawning the `CharacterController` in the `nova-world` Place.
3. **Physics & Control**: Mobile touch inputs from `nova-ui` (virtual joystick, jump) feed into `nova-physics`.
4. **Scripting & Events**: Triggers in `nova-world` execute logic via `nova-scripting`'s sandboxed engine.
5. **Multiplayer Replication**: In online sessions, `nova-network` broadcasts player state and room snapshots.
