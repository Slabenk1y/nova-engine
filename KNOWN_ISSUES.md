# Known Issues & Limitations

1. **Native Display Target in Headless Sandbox**: Direct rendering to an onscreen Android surface requires a connected hardware device or emulator running OpenGL ES 3.0 / Vulkan. The APK contains the compiled and signed activity package ready for device deployment.
2. **Network Transport**: The multiplayer architecture uses high-level packet serialization and authoritative server room models. Running across separate physical devices will require configuring socket bindings or WebSocket gateways.
3. **Audio Output**: Audio architecture is represented through event triggers; direct hardware audio sink integration is scheduled for Phase 5.
