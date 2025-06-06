# Zellij Status Bar Plugin for zjstatus Integration

## Project Overview

This is a modified zellij status bar plugin designed to provide keybinding hints through pipe communication to the zjstatus plugin. The plugin has been significantly refactored from its original multi-line UI implementation to focus on single-line keybinding reference display.

## Project Goals

### Primary Objective
Transform a complex multi-line zellij status bar plugin into a simplified single-line keybinding reference that:
- Shows terse, immediate keybinding hints for the current mode
- Integrates with zjstatus via pipe communication
- Eliminates unnecessary UI complexity while preserving essential functionality

### User Experience Vision
Provide users with an immediate reference showing `<key>:action` mappings for:
- **Normal mode**: Mode-switching keys (pane, tab, resize, move, scroll, search, session)
- **Other modes**: Mode-specific operations plus return to normal
- **Special states**: Priority messages for clipboard operations, errors, fullscreen, floating panes

## Technical Implementation

### Architecture Changes
- **Before**: Complex multi-line system with ~4000+ lines across multiple UI files
- **After**: Simplified single-line system with ~300 lines focused on keybinding display
- **Reduction**: ~94% code reduction while preserving intelligent keybinding detection

### Key Components

#### Core Files
- `src/main.rs` (227 lines): Plugin state management, zjstatus pipe integration, helper functions
- `src/ui.rs` (244 lines): Single-line UI with priority-based display logic
- `src/tip/` (preserved): Intelligent tip system infrastructure (currently unused but maintained)

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

#### Priority Display System
1. **Highest**: Clipboard operation messages
2. **High**: System clipboard errors  
3. **Medium**: Special tab states (fullscreen, floating panes)
4. **Default**: Mode-specific keybinding hints

### Dynamic Keybinding Detection

The plugin intelligently reads the user's actual zellij keybinding configuration and displays only bound keys:

#### Normal Mode Display
Shows available mode-switching keys:
```
<Ctrl+p>:pane <Ctrl+t>:tab <Ctrl+r>:resize <Ctrl+h>:move <Ctrl+s>:scroll <Ctrl+/>:search <Ctrl+o>:session
```

#### Mode-Specific Display
Each mode shows its primary operations:
- **Pane**: `<n>:new <x>:close <w>:float <Enter>:normal`
- **Tab**: `<n>:new <x>:close <Enter>:normal`
- **Resize**: `<+>:grow <->:shrink <Enter>:normal`

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

### Helper Functions
Key helper functions preserved from original implementation:
- `action_key()`: Find keys bound to specific actions
- `action_key_group()`: Find keys for action groups
- `style_key_with_modifier()`: Apply consistent key styling

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