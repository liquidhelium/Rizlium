# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Rizlium is a rhythm game chart editor built in Rust using the Bevy game engine. It's a workspace project with 4 main crates:

- `rizlium_chart`: Core chart data structures and parsing
- `rizlium_render`: Rendering engine for charts
- `rizlium_editor`: Main GUI application with egui
- `rizlium_video_renderer`: Video rendering (currently disabled)

## Development Commands

### Build & Run
```bash
cargo build --release
# Run the editor
cargo run --bin rizlium_editor --release
# Run tests
cargo test
# Check formatting
cargo fmt --check
# Run clippy
cargo clippy
```

### Individual Components
```bash
# Run chart format tests
cargo test -p rizlium_chart
# Run editor only
cargo run -p rizlium_editor
```

### Development Setup
- **MSRV**: rustc 1.76+ (Bevy 0.16 requirement)
- **Backend**: Uses Vulkan on Linux for rendering stability
- **Audio**: Uses bevy_kira_audio with Kira backend

## Architecture

### Core Components

**rizlium_chart/src/**:
- `chart/`: Core data structures (notes, lines, colors, timing)
- `parse/`: Chart format parsers (rizline format)
- `runtime/`: Game state simulation
- `editing/`: Command system for editor operations

**rizlium_render/src/**:
- `notes/`: Note rendering system
- `rings/`: Ring/arc rendering
- `line_rendering/`: Line/bezier curve rendering
- `masks/`: Screen masking system
- `time_and_audio/`: Audio synchronization
- `hit_particles/`: Hit effect particles

**rizlium_editor/src/**:
- `extensions/`: Modular editor panels (timeline, world view, inspector)
- `ui/`: Egui-based interface components
- `project.rs`: Project management
- `settings_module.rs`: Persistent settings

### Key Features

- **Chart Format**: Custom .rzl format with JSON serialization
- **Audio Sync**: Precise audio timing for rhythm games
- **Editor UI**: Dock-based interface with egui_dock
- **Rendering**: 2D Bevy rendering with custom shaders
- **Persistence**: Settings and recent files saved to ~/.config/rizlium-editor/

### File Structure
```
assets/                 # Game assets and sample charts
rizlium_chart/         # Core chart library
rizlium_render/        # Rendering system
rizlium_editor/        # Main application
├── src/extensions/    # Modular editor features
├── src/ui/           # UI components
└── assets/           # Editor assets (fonts, textures)
```

## Development Notes

- **Platform**: Primarily Linux/Wayland, with X11 and webgl2 support
- **Audio Issues**: Wayland screen recording can cause audio state issues (use pipewire)
- **Rendering**: Uses Vulkan backend to avoid OpenGL issues (bevyengine/bevy#10917)
- **Localization**: Chinese/English support via rust-i18n

## Testing

The project uses standard Rust testing. Run with:
```bash
cargo test              # All tests
cargo test -p rizlium_chart  # Chart-specific tests
```

## Dependencies

Key Rust crates:
- **bevy 0.16** - Game engine
- **egui 0.31** - Immediate-mode GUI
- **bevy_egui 0.34** - Egui integration
- **bevy_kira_audio 0.23** - Audio
- **serde** - Serialization
- **rfd** - File dialogs
- **zip** - Archive handling