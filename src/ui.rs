use ansi_term::{Style, Colour::{Fixed, RGB}};
use zellij_tile::prelude::*;
use zellij_tile_utils::palette_match;

use crate::{LinePart};

pub fn text_copied_hint(copy_destination: CopyDestination) -> LinePart {
    let hint = match copy_destination {
        CopyDestination::Command => "Text piped to external command",
        #[cfg(not(target_os = "macos"))]
        CopyDestination::Primary => "Text copied to system primary selection",
        #[cfg(target_os = "macos")] // primary selection does not exist on macos
        CopyDestination::Primary => "Text copied to system clipboard",
        CopyDestination::System => "Text copied to system clipboard",
    };
    LinePart {
        part: serialize_text(&Text::new(&hint).color_range(2, ..).opaque()),
        len: hint.len(),
    }
}

pub fn system_clipboard_error(palette: &Styling) -> LinePart {
    let hint = " Error using the system clipboard.";
    let red_color = palette_match!(palette.text_unselected.emphasis_3);
    LinePart {
        part: Style::new().fg(red_color).bold().paint(hint).to_string(),
        len: hint.len(),
    }
}

// Single-line UI function showing current mode keybindings
pub fn one_line_ui(
    help: &ModeInfo,
    tab_info: Option<&TabInfo>,
    max_len: usize,
    _separator: &str,
    _base_mode_is_locked: bool,
    text_copied_to_clipboard_destination: Option<CopyDestination>,
    clipboard_failure: bool,
    _tip_name: &str,
) -> LinePart {
    // Priority 1: Clipboard messages (highest priority)
    if let Some(text_copied_to_clipboard_destination) = text_copied_to_clipboard_destination {
        return text_copied_hint(text_copied_to_clipboard_destination);
    }
    
    // Priority 2: System clipboard errors
    if clipboard_failure {
        return system_clipboard_error(&help.style.colors);
    }

    // Priority 3: Special tab states (fullscreen, floating panes)
    if let Some(active_tab) = tab_info {
        if active_tab.is_fullscreen_active {
            let text = " FULLSCREEN ";
            let part = serialize_text(&Text::new(text).color_range(1, ..).opaque());
            return LinePart { part, len: text.len() };
        }
        
        if active_tab.are_floating_panes_visible {
            let text = " FLOATING PANES ";
            let part = serialize_text(&Text::new(text).color_range(1, ..).opaque());
            return LinePart { part, len: text.len() };
        }
    }

    // Priority 4: Show current mode keybindings
    show_mode_keybindings(help, max_len)
}

fn show_mode_keybindings(help: &ModeInfo, max_len: usize) -> LinePart {
    use crate::{action_key, style_key_with_modifier};
    use zellij_tile::prelude::actions::{Action, SearchDirection};
    
    let keymap = help.get_mode_keybinds();
    let mut parts = vec![];
    
    match help.mode {
        InputMode::Normal => {
            // Show mode-switching keys from normal mode
            let keybindings = [
                (Action::SwitchToMode(InputMode::Pane), "pane"),
                (Action::SwitchToMode(InputMode::Tab), "tab"), 
                (Action::SwitchToMode(InputMode::Resize), "resize"),
                (Action::SwitchToMode(InputMode::Move), "move"),
                (Action::SwitchToMode(InputMode::Scroll), "scroll"),
                (Action::SwitchToMode(InputMode::Search), "search"),
                (Action::SwitchToMode(InputMode::Session), "session"),
            ];
            
            for (action, label) in keybindings {
                let keys = action_key(&keymap, &[action]);
                if !keys.is_empty() {
                    let styled_keys = style_key_with_modifier(&keys, &help.style.colors, None);
                    parts.extend(styled_keys);
                    parts.push(Style::new().paint(format!(":{} ", label)));
                }
            }
        },
        InputMode::Pane => {
            // Show pane operations
            let keybindings = [
                (Action::NewPane(None, None, false), "new"),
                (Action::CloseFocus, "close"),
                (Action::ToggleFloatingPanes, "float"),
                (Action::SwitchToMode(InputMode::Normal), "normal"),
            ];
            
            for (action, label) in keybindings {
                let keys = action_key(&keymap, &[action]);
                if !keys.is_empty() {
                    let styled_keys = style_key_with_modifier(&keys, &help.style.colors, None);
                    parts.extend(styled_keys);
                    parts.push(Style::new().paint(format!(":{} ", label)));
                }
            }
        },
        InputMode::Tab => {
            // Show tab operations
            let keybindings = [
                (Action::NewTab(None, vec![], None, None, None, false), "new"),
                (Action::CloseTab, "close"),
                (Action::SwitchToMode(InputMode::Normal), "normal"),
            ];
            
            for (action, label) in keybindings {
                let keys = action_key(&keymap, &[action]);
                if !keys.is_empty() {
                    let styled_keys = style_key_with_modifier(&keys, &help.style.colors, None);
                    parts.extend(styled_keys);
                    parts.push(Style::new().paint(format!(":{} ", label)));
                }
            }
        },
        InputMode::Resize => {
            // Show resize operations
            let keybindings = [
                (Action::Resize(Resize::Increase, None), "grow"),
                (Action::Resize(Resize::Decrease, None), "shrink"),
                (Action::SwitchToMode(InputMode::Normal), "normal"),
            ];
            
            for (action, label) in keybindings {
                let keys = action_key(&keymap, &[action]);
                if !keys.is_empty() {
                    let styled_keys = style_key_with_modifier(&keys, &help.style.colors, None);
                    parts.extend(styled_keys);
                    parts.push(Style::new().paint(format!(":{} ", label)));
                }
            }
        },
        InputMode::Move => {
            // Show move operations
            let keybindings = [
                (Action::MovePane(None), "move"),
                (Action::SwitchToMode(InputMode::Normal), "normal"),
            ];
            
            for (action, label) in keybindings {
                let keys = action_key(&keymap, &[action]);
                if !keys.is_empty() {
                    let styled_keys = style_key_with_modifier(&keys, &help.style.colors, None);
                    parts.extend(styled_keys);
                    parts.push(Style::new().paint(format!(":{} ", label)));
                }
            }
        },
        InputMode::Scroll => {
            // Show scroll operations
            let keybindings = [
                (Action::ScrollUp, "up"),
                (Action::ScrollDown, "down"),
                (Action::SwitchToMode(InputMode::Normal), "normal"),
            ];
            
            for (action, label) in keybindings {
                let keys = action_key(&keymap, &[action]);
                if !keys.is_empty() {
                    let styled_keys = style_key_with_modifier(&keys, &help.style.colors, None);
                    parts.extend(styled_keys);
                    parts.push(Style::new().paint(format!(":{} ", label)));
                }
            }
        },
        InputMode::Search => {
            // Show search operations  
            let keybindings = [
                (Action::Search(SearchDirection::Down), "next"),
                (Action::Search(SearchDirection::Up), "prev"),
                (Action::SwitchToMode(InputMode::Normal), "normal"),
            ];
            
            for (action, label) in keybindings {
                let keys = action_key(&keymap, &[action]);
                if !keys.is_empty() {
                    let styled_keys = style_key_with_modifier(&keys, &help.style.colors, None);
                    parts.extend(styled_keys);
                    parts.push(Style::new().paint(format!(":{} ", label)));
                }
            }
        },
        InputMode::Session => {
            // Show session operations
            let keybindings = [
                (Action::Detach, "detach"),
                (Action::SwitchToMode(InputMode::Normal), "normal"),
            ];
            
            for (action, label) in keybindings {
                let keys = action_key(&keymap, &[action]);
                if !keys.is_empty() {
                    let styled_keys = style_key_with_modifier(&keys, &help.style.colors, None);
                    parts.extend(styled_keys);
                    parts.push(Style::new().paint(format!(":{} ", label)));
                }
            }
        },
        _ => {
            // For other modes, just show how to get back to normal
            let keys = action_key(&keymap, &[Action::SwitchToMode(InputMode::Normal)]);
            if !keys.is_empty() {
                let styled_keys = style_key_with_modifier(&keys, &help.style.colors, None);
                parts.extend(styled_keys);
                parts.push(Style::new().paint(":normal "));
            }
        }
    }
    
    if parts.is_empty() {
        return LinePart { part: String::new(), len: 0 };
    }
    
    // Build the final string and calculate length
    use ansi_term::{ANSIStrings, unstyled_len};
    let ansi_strings = ANSIStrings(&parts);
    let formatted = format!(" {}", ansi_strings);
    let len = 1 + unstyled_len(&ansi_strings);
    
    if len <= max_len {
        LinePart { part: formatted, len }
    } else {
        LinePart { part: String::new(), len: 0 }
    }
}