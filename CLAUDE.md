# Zellij Status Bar Plugin for zjstatus Integration

## Project Overview

This is a modified zellij status bar plugin designed to provide keybinding hints through pipe communication to the zjstatus plugin. The plugin has been significantly refactored from its original multi-line UI implementation to focus on single-line keybinding reference display with enhanced visual styling and priority system from the original codebase.

## Project Goals

### Primary Objective
Transform a complex multi-line zellij status bar plugin into a simplified single-line keybinding reference that:
- Shows terse, immediate keybinding hints for the current mode
- Integrates with zjstatus via pipe communication
- Preserves original visual styling and color system
- Maintains intelligent priority-based display logic
- Eliminates unnecessary UI complexity while preserving essential functionality

### User Experience Vision
Provide users with an immediate reference showing segmented `{keybinding} {description}` format:
- **Visual Format**: Saturated background segments for keybindings, less saturated segments for descriptions
- **Normal mode**: Mode-switching keys (pane, tab, resize, move, scroll, search, session, quit)
- **Other modes**: Mode-specific operations plus return to normal
- **Special states**: Enhanced priority messages for clipboard operations, errors, fullscreen, floating panes
- **Modern styling**: Segmented backgrounds with contrasting text, no brackets or colons

## Technical Implementation

### Architecture Changes
- **Before**: Complex multi-line system with ~4000+ lines across multiple UI files
- **After**: Enhanced single-line system with ~750 lines integrating original styling and priority logic
- **Reduction**: ~81% code reduction while preserving intelligent keybinding detection and visual fidelity

### Key Components

#### Core Files
- `src/main.rs` (495 lines): Plugin state management, zjstatus pipe integration, enhanced color system, and helper functions
- `src/ui.rs` (624 lines): Single-line UI with enhanced priority-based display logic and original styling functions
- `src/tip/` (preserved): Intelligent tip system infrastructure (maintained for future integration)

#### Enhanced Features Integration
- **Color System**: Complete `ColoredElements` and `SegmentStyle` structures from original
- **Key Styling**: Full `style_key_with_modifier()` implementation with palette-based theming
- **Priority Functions**: `fullscreen_panes_to_hide()`, `floating_panes_are_visible()`, locked variants
- **Enhanced Tips**: Mode-specific actions including fullscreen, floating, embed, break pane operations

#### Removed Files
- `src/first_line.rs` (1,169 lines): Top-line tab/mode display
- `src/second_line.rs` (804 lines): Complex tip/keybind display logic  
- `src/one_line_ui.rs` (1,604 lines): Original single-line implementation
- `src/common.rs`: Temporary shared types (removed as unused)

### Integration Details

#### zjstatus Pipe Communication
```rust
pipe_message_to_plugin(
    MessageToPlugin::new("pipe")
        .with_payload(format!("zjstatus::pipe::pipe_zjstatus_hints::{}", message)),
);
```
- Sends output to both zjstatus plugin and stdout
- Uses zellij's plugin-to-plugin communication system
- Maintains compatibility with existing zjstatus configuration

#### Enhanced Priority Display System
1. **Highest**: Clipboard operation messages with original styling
2. **High**: System clipboard errors with red color coding
3. **Medium**: Enhanced special tab states with detailed contextual information:
   - `(FULLSCREEN): + N hidden panes` with orange/green color coding
   - `(FLOATING PANES VISIBLE): Press <key>, <key> to hide` with instructional text
   - Locked interface variants: `-- INTERFACE LOCKED --` with appropriate state info
4. **Default**: Enhanced mode-specific keybinding hints with color-coded keys

### Modern Segmented Visual System

#### Segmented Background Styling
- **Keybinding Segments**: Saturated background (`palette.ribbon_unselected.background`) with contrasting foreground
- **Description Segments**: Less saturated background (`palette.text_unselected.background`) with contrasting foreground
- **No Brackets**: Keys displayed directly without `<>` wrapping
- **No Colons**: Descriptions follow keybindings without `:` separators
- **Clean Separation**: Visual separation through background color contrast

#### Enhanced Key Display Intelligence
- **Common Modifiers**: `Ctrl + p|t|r` format when keys share modifiers (no brackets)
- **Special Groups**: `hjkl`, `←↓↑→` displayed without separators in single segment
- **Mixed Modifiers**: `Alt a|Ctrl b|c` when keys have different modifiers (no brackets)
- **Segmented Layout**: Each `{keybinding} {description}` pair forms distinct visual units

### Dynamic Keybinding Detection

The plugin intelligently reads the user's actual zellij keybinding configuration and displays only bound keys:

#### Normal Mode Display
Shows available mode-switching keys with modern segmented styling:
```
{Ctrl+p} {pane} {Ctrl+t} {tab} {Ctrl+r} {resize} {Ctrl+h} {move} {Ctrl+s} {scroll} {Ctrl+/} {search} {Ctrl+o} {session} {Ctrl+q} {quit}
```

#### Enhanced Mode-Specific Display
Each mode shows comprehensive operations with segmented background styling:
- **Pane**: `{n} {new} {x} {close} {f} {fullscreen} {w} {floating} {e} {embed} {Enter} {select}`
- **Tab**: `{n} {new} {x} {close} {h} {prev} {l} {next} {b} {break pane} {Enter} {select}`
- **Resize**: `{+} {increase} {-} {decrease} {Enter} {select}`
- **Scroll**: `{/} {search} {j} {scroll} {d} {page} {u} {half page} {e} {edit} {Enter} {select}`
- **Search**: `{/} {search} {n} {down} {N} {up} {Enter} {select}`

## Build Process

### Target Platform
- **Target**: `wasm32-wasip1` (WebAssembly for zellij plugins)
- **Build command**: `cargo build --target=wasm32-wasip1`
- **Dependencies**: Standard zellij plugin dependencies (zellij-tile, ansi-term)

### Development Workflow
1. Test compilation frequently during refactoring
2. Maintain plugin interface compatibility
3. Preserve helper functions for potential tip system restoration
4. Use git history to understand original behavior patterns

## Future Considerations

### Tip System Infrastructure
The original intelligent tip system (`src/tip/`) has been preserved but is currently unused. This system includes:
- Dynamic tip rotation and caching
- Context-aware keybinding analysis
- Multi-format tip display (short/medium/full)

This infrastructure could be re-integrated if sentence-based tips are desired in the future, though current focus is on immediate keybinding reference.

### Potential Enhancements
- Adaptive display based on terminal width
- Custom keybinding groupings
- Integration with zellij's theme system
- Additional special state detection

## Development Notes

### Enhanced Helper Functions
Key helper functions restored and enhanced from original implementation:
- `action_key()`: Find keys bound to specific actions
- `action_key_group()`: Find keys for action groups  
- `style_key_with_modifier()`: Modern segmented styling with saturated backgrounds for keybindings
- `style_description()`: Segmented styling with less saturated backgrounds for descriptions
- `get_common_modifiers()`: Detect shared modifiers across key groups
- `color_elements()`: Generate complete color theming system
- Priority display functions: `fullscreen_panes_to_hide()`, `floating_panes_are_visible()`, etc.

### Testing Approach
- Verify compilation after each major change
- Test with various terminal widths
- Validate keybinding detection accuracy
- Ensure zjstatus pipe integration works correctly

### Code Style
- Minimal commenting (as requested)
- Focus on functionality over documentation
- Consistent with existing zellij plugin patterns
- Single-line focus throughout UI logic
- Preserves original visual consistency and user experience

## Recent Enhancements (Latest Update)

### Modern Segmented Styling Implementation
- **Segmented Background System**: Completely redesigned visual styling with contrasting background segments
- **Removed Legacy Elements**: Eliminated `<>` brackets and `:` separators for cleaner appearance
- **Dual Background Colors**: Saturated backgrounds for keybindings, less saturated for descriptions
- **Enhanced Readability**: Clear visual separation through background color contrast instead of punctuation

### Styling Function Modernization
- **style_key_with_modifier()**: Redesigned to create saturated background segments for keybindings
- **style_description()**: New function creating less saturated background segments for action descriptions
- **Palette Integration**: Uses zellij's theme system with `ribbon_unselected` and `text_unselected` palettes
- **Format Consistency**: All hints follow `{keybinding} {description}` segmented pattern

### Visual Experience Enhancement
- **Modern UI Paradigm**: Adopts contemporary segmented styling approach similar to modern status bars
- **Improved Contrast**: Better text readability through background/foreground color optimization
- **Consistent Spacing**: Uniform padding within each segment for visual balance
- **Theme Compatibility**: Automatically adapts to different zellij color themes

### Comprehensive Mode Coverage
- **All Input Modes**: Complete keybinding coverage across Normal, Pane, Tab, Resize, Move, Scroll, Search, Session modes
- **Enhanced Pane Actions**: Includes fullscreen, floating, embed operations with proper key detection
- **Advanced Operations**: Tab navigation, break pane, sync, rename functionality
- **Scroll/Search Integration**: Complete scroll mode with search, paging, and edit operations

The plugin now provides a modern, visually clean interface while maintaining the comprehensive functionality and intelligent keybinding detection of the original implementation, optimized for zjstatus integration.