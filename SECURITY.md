# Security Model & Policy

## Authentication & Passwords
- Passwords are encrypted using SHA-256 cryptographic hashing prior to storage.
- Plaintext passwords are never persisted.

## Scripting Sandboxing
- The visual scripting runtime enforces a strict instruction execution limit per tick (`max_instructions_per_tick = 5000`) to prevent infinite loop execution and CPU denial-of-service.
- Untrusted user scripts operate strictly on isolated `ScriptExecutionContext` structures without arbitrary filesystem or OS process execution permissions.

## Authoritative Multiplayer
- Important game state transitions (character movement, scoring, inventory modifications, place changes) are validated server-side.
- Client packets are treated as requests and validated against physics constraints before mutating room snapshots.
