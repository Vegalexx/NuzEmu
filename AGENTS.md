# AGENTS.md

## 1. Project vision

NuzEmu is an all-in-one Pokémon desktop companion built with Angular, Tauri and Rust.

The long-term goal is to combine:

- A game library with metadata, artwork and session management.
- An embedded emulator with reliable controls and save persistence.
- A Pokémon companion with live party information, detailed Pokémon profiles and a Pokédex.
- Support for multiple Pokémon games and generations through a modular architecture.

### Product direction

- **Product:** all-in-one application, combining a game launcher, emulator and Pokémon companion.
- **Visual identity:** modern gaming interface with subtle retro and pixel-art elements.
- **Compatibility:** multi-game and multi-generation architecture.
- **Initial reference game:** Pokémon Émeraude FR.
- **Platform:** Windows desktop initially.

Build a reliable experience for the current game before expanding compatibility. Do not assume that memory layouts, species identifiers, save formats or game features are identical across games.

## 2. Current technology

- Frontend: Angular, TypeScript, HTML and SCSS.
- Desktop runtime: Tauri 2.
- Backend: Rust.
- Emulation: mGBA Libretro core.
- Party data: direct reading of the embedded core's system RAM, with a TCP/Lua prototype fallback.

## 3. Development commands

Run from the project root:

```bash
npm install
npm run tauri dev
```

Check the Rust backend:

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Review changes before committing:

```bash
git status
git diff --check
git diff
```

Run relevant checks after changes. Existing tests may be incomplete during the proof-of-concept phase. Clearly report which checks were run and which were skipped.

## 4. Project structure

### Angular

- `src/app/app.ts`: root component.
- `src/app/app.html`: root template.
- `src/app/app.scss`: global application styling.
- `src/app/core/tauri.ts`: Tauri command wrappers and shared TypeScript types.
- `src/app/features/emulator/emulator.ts`: emulator lifecycle, frame loop and keyboard controls.
- `src/app/features/emulator/emulator.html`: emulator interface.
- `src/app/features/emulator/emulator.scss`: emulator styling.
- `src/app/features/party/party.ts`: party snapshot polling and presentation logic.
- `src/app/features/party/party.html`: party interface.
- `src/app/features/party/party.scss`: party styling.

### Rust and resources

- `src-tauri/src/lib.rs`: Tauri commands, application lifecycle and TCP bridge.
- `src-tauri/src/libretro.rs`: Libretro loading, ROM execution, input, video, save RAM and party decoding.
- `src-tauri/resources/cores/mgba_libretro.dll`: mGBA Libretro core.
- `games/roms/PokemonEmeraudeFR.gba`: local ROM.
- `games/saves/PokemonEmeraudeFR.sav`: local save file.
- `scripts/mgba/`: Lua prototype scripts.

This is the current structure, not a requirement to keep all future functionality in these files. Follow the roadmap when introducing new modules.

## 5. Emulator behavior

Keep the emulator embedded inside NuzEmu. Do not replace it with a separate mGBA application.

The frontend runs the emulator frame loop and displays the returned RGBA frame in a canvas.

Current keyboard controls:

- `Z`: Up.
- `Q`: Left.
- `S`: Down.
- `D`: Right.
- `A`: L shoulder button.
- `E`: R shoulder button.
- `L`: B.
- `M`: A.
- `Enter`: Start.
- `Backspace`: Select.
- `Tab`: cycle speed x1, x2, x4.
- `P`: start or pause.
- `R`: reset the ROM.

When changing keyboard handling:

- Preserve key-down and key-up behavior.
- Prevent browser defaults for game controls when necessary, including Backspace.
- Do not intercept normal text editing inside text inputs or text areas.
- Keep the existing keyboard mappings unless a change is explicitly requested.

The reset operation must wait for any in-progress frame operation before shutting down and rebooting the core.

## 6. Save persistence

The existing `.sav` file must be loaded into the Libretro core at startup.

Pokémon's own in-game save system remains authoritative. Persist the core's exposed save RAM when changes settle and during clean application shutdown.

Implementation details:

- `RETRO_MEMORY_SAVE_RAM` identifies save RAM.
- Change detection uses `observed_save_ram`, `save_dirty` and `frames_since_save_change`.
- The current stabilization threshold is 30 unchanged frames.
- `shutdown_core()` must persist save RAM before destroying the core.
- Application exit handling must call `shutdown_core()`.
- ROM reset must shut down the current core cleanly before booting it again.

Do not add periodic disk writes or a separate manual save button unless explicitly requested.

When changing save handling, test loading, in-game saving, reset, normal shutdown and reopening the same game.

## 7. Party integration and memory safety

The party panel should reflect the game running inside NuzEmu, not a separate emulator process.

The primary path reads party data from the embedded Libretro core's system RAM. The TCP/Lua bridge is a prototype fallback.

The current Tauri command `get_party_snapshot` prefers embedded-core data and can fall back to the TCP snapshot.

Avoid logging the same expected fallback error on every frontend poll.

### Current Pokémon Émeraude layout

The current decoder uses these GBA addresses:

- Party count: `0x020244E9`.
- Party data: `0x020244EC`.
- Party record size: 100 bytes.

Keep the distinction between absolute GBA addresses and offsets relative to the RAM pointer explicit.

### Memory safety requirements

The current core has reported a 32 KiB system RAM size, while the party decoder has required a larger readable range.

Treat this as an unresolved core-specific assumption until the actual accessible memory region is verified.

- Never assume a pointer is safe to read beyond the size reported by the API without verifying the allocation.
- Validate pointer availability and memory bounds before decoding.
- Fail gracefully if the memory layout is unknown or unavailable.
- Do not silently decode invalid memory as valid Pokémon data.
- Log actionable errors without flooding the console.

Correctness and memory safety take priority over temporarily displaying party information.

## 8. Pokémon species and game-specific data

Species identifiers read from game memory are internal game identifiers. They must not automatically be treated as National Pokédex numbers.

The current `species_info()` function in `src-tauri/src/libretro.rs` maps internal identifiers to French names and National Pokédex numbers.

When adding species:

- Verify the internal identifier against a reliable reference for the exact game.
- Use the correct French name.
- Store the National Pokédex number separately.
- Preserve an unknown-species fallback for genuinely unmapped values.
- Avoid assuming that a mapping valid for one game or generation is valid for another.

As the architecture evolves, move game-specific species data into a dedicated data module or resource rather than expanding a large hard-coded match statement indefinitely.

Keep presentation data separate from memory-decoding logic.

## 9. Architecture principles for multi-game support

The emulator engine and game-specific interpretation must be separate.

### Shared emulator responsibilities

- Loading and unloading a compatible core.
- ROM lifecycle and input handling.
- Frame execution and video output.
- Audio integration when implemented.
- Save RAM access and persistence.
- Generic session status and error handling.

### Game-specific responsibilities

- Game identification and metadata.
- Memory layouts and party decoding.
- Species identifiers and localized names.
- Game-specific save interpretation.
- Available companion features and their support status.

Introduce game adapters incrementally. The first adapter should be Pokémon Émeraude.

Potential future modules include:

- `games/metadata.rs`: common game metadata.
- `games/pokemon/emerald.rs`: Émeraude-specific memory decoding and data.
- A shared adapter interface for exposing supported game information.

These are architectural targets, not instructions to move every existing file immediately.

Do not introduce an abstraction without a concrete use case. Keep each refactoring small and verify that Émeraude still works before adding another game.

## 10. Visual identity and UX

NuzEmu should combine a modern desktop experience with restrained retro Pokémon-inspired details.

Design principles:

- Clear hierarchy and readable typography.
- Dark gaming surfaces with controlled accent colors.
- Pixel-art sprites rendered sharply.
- Consistent spacing, component sizes and interaction states.
- Subtle transitions rather than distracting animations.
- Responsive layouts for different window sizes.
- Clear loading, empty, disconnected and error states.

The intended top-level areas are:

- Library.
- In-game emulator.
- Pokémon companion.
- Settings.

Do not sacrifice usability or readability for retro styling.

## 11. Roadmap and priorities

The roadmap defines the product direction. Complete and validate the current milestone before committing to major work in a later milestone.

### Milestone 1: Reliable Pokémon data and emulator foundation

**Priority: immediate**

- Complete the 386 species of Generation III with French names and National Pokédex numbers.
- Validate party species, levels, current HP and maximum HP.
- Test captures, party rearrangement, evolution and fainted Pokémon.
- Verify the actual readable RAM region and all memory boundaries.
- Validate save loading, in-game saving, reset and shutdown.
- Eliminate repeated expected errors in logs.

**Acceptance criteria:**

- Party information matches the running game.
- Unsupported or invalid data is handled safely.
- Normal shutdown and reopening preserve the save.
- Memory reads respect verified bounds.

### Milestone 2: Introduce the game adapter architecture

**Priority: high**

- Define common game metadata.
- Separate generic emulator operations from game-specific data.
- Introduce the first game adapter for Pokémon Émeraude.
- Isolate memory offsets, species mappings and decoding rules.
- Keep Angular components independent of game-specific memory layouts.
- Preserve existing emulator behavior during the refactor.

**Acceptance criteria:**

- Émeraude continues to work without regressions.
- Game-specific decoding is not embedded in generic emulator code.
- A second game could be added without rewriting the frontend.

### Milestone 3: Build the game library

**Priority: high**

- Add a library screen with titles, artwork and platform information.
- Allow users to select local game files or directories.
- Identify supported games where reliable metadata is available.
- Associate saves and settings with the correct game.
- Add launch, resume and remove-from-library actions.
- Handle missing, invalid and unsupported files clearly.

**Acceptance criteria:**

- A user can add and launch a supported game.
- The game can be reopened with its own save and settings.
- Unsupported files produce clear explanations.

Use game files supplied by the user. Do not bundle copyrighted commercial ROMs with the application.

### Milestone 4: Establish the modern-retro design system

**Priority: medium-high**

- Add consistent navigation between Library, In-game, Companion and Settings.
- Establish reusable colors, typography, spacing and component styles.
- Improve Pokémon party cards and sprite rendering.
- Add clear session status and error feedback.
- Make the application comfortable at different window sizes.

**Acceptance criteria:**

- All main screens share a consistent visual identity.
- The emulator remains the focal point during gameplay.
- Loading and error states are understandable without developer tools.

### Milestone 5: Expand companion features

**Priority: medium**

- Add detailed Pokémon profiles.
- Show types, stats, moves and held items when reliably available.
- Introduce a searchable Pokédex.
- Add seen and caught tracking only when the required data can be obtained reliably.
- Add evolution and type information.
- Keep game-specific datasets separate.

**Acceptance criteria:**

- Selecting a party member opens a correct, useful profile.
- The companion uses data appropriate to the active game and version.
- Missing information is clearly distinguished from a confirmed absence.

### Milestone 6: Add another game and prepare distribution

**Priority: later**

- Select a second supported Pokémon game to validate the adapter architecture.
- Add its own adapter and tested data mappings.
- Introduce clear compatibility and version reporting.
- Prepare a Windows installation package and application metadata.
- Review redistribution licenses for emulator cores and other bundled resources.
- Add useful diagnostic information for end users.

**Acceptance criteria:**

- The second game works through the same shared application architecture.
- A clean installation can launch a supported user-provided game.
- Unsupported features and compatibility limitations are documented.

## 12. Features deliberately deferred

Unless explicitly requested, defer:

- Cloud save synchronization.
- User accounts and social features.
- Save editing.
- Third-party plugin installation.
- Support for every Pokémon generation in the first release.

First establish reliable game data, safe memory access, game adapters and save lifecycle handling.

## 13. Working method

For each task:

1. Inspect `git status` and the existing diff.
2. Read the relevant source files and callers.
3. Identify whether the behavior is generic or game-specific.
4. Make the smallest safe change.
5. Run relevant frontend or Rust checks.
6. Test emulator controls, reset and saves if affected.
7. Verify party data and memory bounds if affected.
8. Review the diff for unrelated changes.
9. Report modified files, checks actually performed and remaining limitations.

Keep Angular features split across their existing TypeScript, HTML and SCSS files.

Follow current naming conventions and avoid introducing dependencies without justification.

Do not claim that a build or test passed unless it was actually run successfully.
