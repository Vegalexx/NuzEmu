# AGENTS.md

## Project overview

**NuzEmu** is a Pokémon desktop companion built with Angular and Tauri. It includes a Libretro-based emulator prototype using the mGBA core and a party-tracking feature.

The project currently targets Windows 11.

## Technology stack

- Angular 22
- TypeScript
- Tauri 2
- Rust
- Libretro API
- mGBA Libretro core
- SCSS

## Project structure

```text
NuzEmu/
├── src/
│   └── app/
│       ├── app.ts
│       ├── app.html
│       ├── app.scss
│       ├── core/
│       │   └── tauri.ts
│       └── features/
│           ├── emulator/
│           │   ├── emulator.ts
│           │   ├── emulator.html
│           │   └── emulator.scss
│           └── party/
│               ├── party.ts
│               ├── party.html
│               └── party.scss
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs
│   │   └── libretro.rs
│   └── resources/
│       └── cores/
│           └── mgba_libretro.dll
├── games/
│   ├── roms/
│   └── saves/
└── AGENTS.md
```

Angular component filenames use the project's existing naming convention, such as `emulator.ts` and `emulator.html`. Do not assume filenames like `emulator.component.ts` or `tauri.service.ts`.

## Development commands

Run the desktop application:

```bash
npm run tauri dev
```

Check the Rust backend:

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Run Angular checks or builds using the scripts defined in `package.json`.

## Architecture and conventions

- Keep UI features separated into focused Angular components.
- Keep emulator UI and input handling in `src/app/features/emulator/`.
- Keep Tauri bridge calls in `src/app/core/tauri.ts`.
- Keep Libretro core management and emulator-specific native logic in `src-tauri/src/libretro.rs`.
- Keep Tauri commands and application lifecycle wiring in `src-tauri/src/lib.rs`.
- Reuse existing patterns and dependencies before introducing new abstractions or packages.
- Keep the interface in French.
- Avoid unrelated refactors when implementing a focused change.
- Update this file when the architecture or important behavior changes.

## Emulator implementation

The emulator prototype loads the mGBA Libretro DLL and a Pokémon Game Boy Advance ROM.

- The video frame is 240 × 160 pixels.
- Rust exposes frame data to Angular as Base64-encoded RGBA pixels.
- Angular renders the image on a canvas.
- The Angular emulator loop requests frames from the Rust backend.
- The emulator supports starting, pausing, resuming, speed changes and ROM reset.
- Speed options are x1, x2 and x4.

### Keyboard controls

| Key | Action |
|---|---|
| Z | Up |
| Q | Left |
| S | Down |
| D | Right |
| A | L shoulder |
| E | R shoulder |
| L | B |
| M | A |
| Enter | Start |
| Delete | Select |
| Tab | Cycle speed: x1 → x2 → x4 → x1 |
| P | Toggle play/pause |
| R | Reset the ROM |

Keyboard shortcuts must use the same underlying methods as their corresponding UI buttons. In particular, the Play/Pause button and the `P` shortcut both use `toggleGame()`.

Ignore emulator shortcuts when the user is typing in an input, textarea, select or contenteditable element. Prevent browser-default behavior when necessary, and avoid repeating actions while a key is held down.

Ensure pressed emulator keys are released when appropriate, including on keyup and window blur.

## Save-game persistence

The in-game Pokémon save mechanism remains authoritative.

- Load an existing `.sav` file into Libretro save RAM when booting the ROM.
- Read save RAM through `RETRO_MEMORY_SAVE_RAM`.
- Detect changes by comparing the current save RAM against an observed snapshot.
- After save RAM changes remain stable for the configured number of frames, write the updated data to the `.sav` file.
- Persist save RAM during clean application shutdown.
- Shut down the core before restarting the ROM to avoid leaving the previous core instance active.

Do not add periodic disk writes every fixed number of frames or a separate manual-save button unless explicitly requested.

The current save-change detection monitors memory changes, not the Pokémon game's save-menu event directly. Avoid claiming that the application detects the in-game save event itself.

## Emulator lifecycle and safety

- Avoid overlapping frame requests.
- Wait for an active frame operation to finish before resetting or shutting down the core.
- Preserve the existing synchronization and error-handling mechanisms.
- Release resources and shut down the Libretro core when the application exits.
- Avoid unnecessary nested `unsafe` blocks inside an existing `unsafe` scope.
- Do not remove the existing Lua prototype scripts or the TCP listener on `127.0.0.1:18765`; they support the party-data prototype.

## Current prototype assets

- mGBA core: `src-tauri/resources/cores/mgba_libretro.dll`
- Pokémon Émeraude French ROM: `games/roms/PokemonEmeraudeFR.gba`
- Pokémon Émeraude save: `games/saves/PokemonEmeraudeFR.sav`

Treat ROMs and save files as user data. Do not overwrite or delete them as part of unrelated code changes.

## Before committing

1. Inspect `git status`.
2. Run the relevant checks for the modified code.
3. Review the diff for accidental changes to ROMs, saves or unrelated files.
4. Use a concise commit message describing the change.
5. Do not commit generated files or game assets unless explicitly requested.