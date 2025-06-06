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
Provide users with an immediate reference showing `<key>:action` mappings for:
- **Normal mode**: Mode-switching keys (pane, tab, resize, move, scroll, search, session, quit)
- **Other modes**: Mode-specific operations plus return to normal
- **Special states**: Enhanced priority messages for clipboard operations, errors, fullscreen, floating panes
- **Visual consistency**: Color-coded keys and modifiers matching original theme system

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

### Enhanced Visual System

#### Color Coding
- **Orange**: Key modifiers (Ctrl, Alt) and status indicators (FULLSCREEN, FLOATING PANES)
- **Green**: Individual keys and numeric values
- **Regular text**: Brackets, separators, and descriptive text
- **Red**: Error states (clipboard failures)
- **Dimmed/Italic**: Disabled states

#### Key Display Intelligence
- **Common Modifiers**: `Ctrl + <p|t|r>` format when keys share modifiers
- **Special Groups**: `<hjkl>`, `<←↓↑→>` displayed without separators
- **Mixed Modifiers**: `<Alt a|Ctrl b|c>` when keys have different modifiers

### Dynamic Keybinding Detection

The plugin intelligently reads the user's actual zellij keybinding configuration and displays only bound keys:

#### Normal Mode Display
Shows available mode-switching keys with enhanced color coding:
```
<Ctrl+p>:pane <Ctrl+t>:tab <Ctrl+r>:resize <Ctrl+h>:move <Ctrl+s>:scroll <Ctrl+/>:search <Ctrl+o>:session <Ctrl+q>:quit
```

#### Enhanced Mode-Specific Display
Each mode shows comprehensive operations with original action set:
- **Pane**: `<n>:new <x>:close <f>:fullscreen <w>:floating <e>:embed <Enter>:normal`
- **Tab**: `<n>:new <x>:close <h>:prev <l>:next <b>:break pane <Enter>:normal`
- **Resize**: `<+>:+ <->:- <Enter>:normal`
- **Scroll**: `</>:search <j>:down <k>:up <d>:page down <u>:page up <e>:edit <Enter>:normal`
- **Search**: `</>:search <n>:next <N>:prev <Enter>:normal`

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
- `style_key_with_modifier()`: Full original implementation with palette-based theming
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

### Visual Fidelity Restoration
- **Complete Color System**: Integrated original `ColoredElements` and `SegmentStyle` structures
- **Palette Integration**: Uses zellij's theme system for consistent colors across different themes
- **Enhanced Key Styling**: Restored sophisticated key grouping and modifier handling

### Priority System Enhancement
- **Rich Context Messages**: Fullscreen and floating pane states show detailed, actionable information
- **Intelligent State Handling**: Different behavior for Normal/Locked vs other modes
- **Visual Consistency**: Maintains original color coding and formatting

### Expanded Action Coverage
- **Comprehensive Mode Support**: All modes now show relevant actions from original implementation
- **Enhanced Pane Actions**: Includes fullscreen, floating, embed operations with proper key detection
- **Tab Operations**: Added navigation (prev/next) and break pane functionality
- **Scroll Enhancements**: Complete scroll mode with search, paging, and edit operations

The plugin now provides the full visual experience of the original while maintaining the streamlined single-line architecture optimized for zjstatus integration.