use ansi_term::{
    Colour::{Fixed, RGB},
    Style,
};
use zellij_tile::prelude::actions::Action;
use zellij_tile::prelude::*;
use zellij_tile_utils::palette_match;

use crate::{
    action_key, action_key_group, single_action_key, style_description, LinePart, TO_NORMAL,
};

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

// Single-line UI function showing current mode keybindings with enhanced priority system
pub fn one_line_ui(
    help: &ModeInfo,
    tab_info: Option<&TabInfo>,
    max_len: usize,
    _separator: &str,
    base_mode_is_locked: bool,
    text_copied_to_clipboard_destination: Option<CopyDestination>,
    clipboard_failure: bool,
    tip_name: &str,
) -> LinePart {
    // Priority 1: Clipboard messages (highest priority)
    if let Some(text_copied_to_clipboard_destination) = text_copied_to_clipboard_destination {
        return text_copied_hint(text_copied_to_clipboard_destination);
    }

    // Priority 2: System clipboard errors
    if clipboard_failure {
        return system_clipboard_error(&help.style.colors);
    }

    // Priority 3: Special tab states with enhanced messages from original
    if let Some(active_tab) = tab_info {
        if active_tab.is_fullscreen_active {
            match help.mode {
                InputMode::Normal => {
                    return fullscreen_panes_to_hide(&help.style.colors, active_tab.panes_to_hide)
                }
                InputMode::Locked => {
                    return locked_fullscreen_panes_to_hide(
                        &help.style.colors,
                        active_tab.panes_to_hide,
                    )
                }
                _ => {
                    // For other modes, show simple fullscreen indicator and continue to keybindings
                    if max_len > 20 {
                        let mut result = show_mode_keybindings(help, max_len - 15);
                        if result.len > 0 {
                            result.part = format!(" (FULLSCREEN) {}", result.part.trim_start());
                            result.len += 14; // " (FULLSCREEN) "
                        } else {
                            let text = " (FULLSCREEN) ";
                            let orange_color =
                                palette_match!(help.style.colors.text_unselected.emphasis_0);
                            result = LinePart {
                                part: Style::new().fg(orange_color).bold().paint(text).to_string(),
                                len: text.len(),
                            };
                        }
                        return result;
                    }
                }
            }
        }

        if active_tab.are_floating_panes_visible {
            match help.mode {
                InputMode::Normal => return floating_panes_are_visible(help),
                InputMode::Locked => return locked_floating_panes_are_visible(&help.style.colors),
                _ => {
                    // For other modes, show simple floating panes indicator and continue to keybindings
                    if max_len > 25 {
                        let mut result = show_mode_keybindings(help, max_len - 20);
                        if result.len > 0 {
                            result.part = format!(" (FLOATING PANES) {}", result.part.trim_start());
                            result.len += 19; // " (FLOATING PANES) "
                        } else {
                            let text = " (FLOATING PANES) ";
                            let orange_color =
                                palette_match!(help.style.colors.text_unselected.emphasis_0);
                            result = LinePart {
                                part: Style::new().fg(orange_color).bold().paint(text).to_string(),
                                len: text.len(),
                            };
                        }
                        return result;
                    }
                }
            }
        }
    }

    // Priority 4: Show current mode keybindings with tips from original system
    match help.mode {
        InputMode::Locked if base_mode_is_locked => LinePart {
            part: String::new(),
            len: 0,
        },
        _ => show_enhanced_mode_keybindings(help, max_len, tip_name),
    }
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
                    let styled_desc = style_description(label, &help.style.colors);
                    parts.extend(styled_desc);
                }
            }
        }
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
                    let styled_desc = style_description(label, &help.style.colors);
                    parts.extend(styled_desc);
                }
            }
        }
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
                    parts.push(Style::new().paint(format!(" {} ", label)));
                }
            }
        }
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
                    parts.push(Style::new().paint(format!(" {} ", label)));
                }
            }
        }
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
                    parts.push(Style::new().paint(format!(" {} ", label)));
                }
            }
        }
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
                    parts.push(Style::new().paint(format!(" {} ", label)));
                }
            }
        }
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
                    parts.push(Style::new().paint(format!(" {} ", label)));
                }
            }
        }
        InputMode::Session => {
            // Use original implementation pattern for session operations
            let detach_keys = action_key(&keymap, &[Action::Detach]);
            if !detach_keys.is_empty() {
                let styled_keys = style_key_with_modifier(&detach_keys, &help.style.colors, None);
                parts.extend(styled_keys);
                let styled_desc = style_description("detach", &help.style.colors);
                parts.extend(styled_desc);
            }

            // Select pane (return to normal)
            let to_normal_keys = action_key(&keymap, &[TO_NORMAL]);
            let select_key = if to_normal_keys.contains(&KeyWithModifier::new(BareKey::Enter)) {
                vec![KeyWithModifier::new(BareKey::Enter)]
            } else {
                to_normal_keys.into_iter().take(1).collect()
            };
            if !select_key.is_empty() {
                let styled_keys = style_key_with_modifier(&select_key, &help.style.colors, None);
                parts.extend(styled_keys);
                let styled_desc = style_description("select", &help.style.colors);
                parts.extend(styled_desc);
            }
        }
        _ => {
            // For other modes, just show how to get back to normal
            let keys = action_key(&keymap, &[Action::SwitchToMode(InputMode::Normal)]);
            if !keys.is_empty() {
                let styled_keys = style_key_with_modifier(&keys, &help.style.colors, None);
                parts.extend(styled_keys);
                let styled_desc = style_description("normal", &help.style.colors);
                parts.extend(styled_desc);
            }
        }
    }

    if parts.is_empty() {
        return LinePart {
            part: String::new(),
            len: 0,
        };
    }

    // Build the final string and calculate length
    use ansi_term::{unstyled_len, ANSIStrings};
    let ansi_strings = ANSIStrings(&parts);
    let formatted = format!("{}", ansi_strings);
    let len = unstyled_len(&ansi_strings);

    if len <= max_len {
        LinePart {
            part: formatted,
            len,
        }
    } else {
        LinePart {
            part: String::new(),
            len: 0,
        }
    }
}

pub fn fullscreen_panes_to_hide(palette: &Styling, panes_to_hide: usize) -> LinePart {
    let text_color = palette_match!(palette.text_unselected.base);
    let green_color = palette_match!(palette.text_unselected.emphasis_2);
    let orange_color = palette_match!(palette.text_unselected.emphasis_0);
    let shortcut_left_separator = Style::new().fg(text_color).bold().paint(" (");
    let shortcut_right_separator = Style::new().fg(text_color).bold().paint("): ");
    let fullscreen = "FULLSCREEN";
    let plus = "+ ";
    let panes = panes_to_hide.to_string();
    let hide = " hidden panes";
    let len = fullscreen.chars().count()
        + plus.chars().count()
        + panes.chars().count()
        + hide.chars().count()
        + 5; // 3 for ():'s around shortcut, 2 for the space
    LinePart {
        part: format!(
            "{}{}{}{}{}{}",
            shortcut_left_separator,
            Style::new().fg(orange_color).bold().paint(fullscreen),
            shortcut_right_separator,
            Style::new().fg(text_color).bold().paint(plus),
            Style::new().fg(green_color).bold().paint(panes),
            Style::new().fg(text_color).bold().paint(hide)
        ),
        len,
    }
}

pub fn floating_panes_are_visible(mode_info: &ModeInfo) -> LinePart {
    let palette = mode_info.style.colors;
    let km = &mode_info.get_mode_keybinds();
    let white_color = palette_match!(palette.text_unselected.base);
    let green_color = palette_match!(palette.text_unselected.emphasis_2);
    let orange_color = palette_match!(palette.text_unselected.emphasis_0);
    let shortcut_left_separator = Style::new().fg(white_color).bold().paint(" (");
    let shortcut_right_separator = Style::new().fg(white_color).bold().paint("): ");
    let floating_panes = "FLOATING PANES VISIBLE";
    let press = "Press ";
    let pane_mode = format!(
        "{}",
        action_key(km, &[Action::SwitchToMode(InputMode::Pane)])
            .first()
            .unwrap_or(&KeyWithModifier::new(BareKey::Char('?')))
    );
    let plus = ", ";
    let p_left_separator = "<";
    let p = format!(
        "{}",
        action_key(
            &mode_info.get_keybinds_for_mode(InputMode::Pane),
            &[Action::ToggleFloatingPanes, TO_NORMAL]
        )
        .first()
        .unwrap_or(&KeyWithModifier::new(BareKey::Char('?')))
    );
    let p_right_separator = "> ";
    let to_hide = "to hide.";

    let len = floating_panes.chars().count()
        + press.chars().count()
        + pane_mode.chars().count()
        + plus.chars().count()
        + p_left_separator.chars().count()
        + p.chars().count()
        + p_right_separator.chars().count()
        + to_hide.chars().count()
        + 5; // 3 for ():'s around floating_panes, 2 for the space
    LinePart {
        part: format!(
            "{}{}{}{}{}{}{}{}{}{}",
            shortcut_left_separator,
            Style::new().fg(orange_color).bold().paint(floating_panes),
            shortcut_right_separator,
            Style::new().fg(white_color).bold().paint(press),
            Style::new().fg(green_color).bold().paint(pane_mode),
            Style::new().fg(white_color).bold().paint(plus),
            Style::new().fg(white_color).bold().paint(p_left_separator),
            Style::new().fg(green_color).bold().paint(p),
            Style::new().fg(white_color).bold().paint(p_right_separator),
            Style::new().fg(white_color).bold().paint(to_hide),
        ),
        len,
    }
}

pub fn locked_fullscreen_panes_to_hide(palette: &Styling, panes_to_hide: usize) -> LinePart {
    LinePart {
        part: String::new(),
        len: 0,
    }
}

pub fn locked_floating_panes_are_visible(palette: &Styling) -> LinePart {
    LinePart {
        part: String::new(),
        len: 0,
    }
}

// Enhanced mode keybindings that includes better tips and actions from the original
fn show_enhanced_mode_keybindings(help: &ModeInfo, max_len: usize, _tip_name: &str) -> LinePart {
    use crate::{action_key, style_key_with_modifier};
    use zellij_tile::prelude::actions::{Action, SearchDirection};

    let keymap = match help.mode {
        InputMode::Normal => help.get_keybinds_for_mode(InputMode::Normal),
        InputMode::Pane => help.get_keybinds_for_mode(InputMode::Pane),
        InputMode::Tab => help.get_keybinds_for_mode(InputMode::Tab),
        InputMode::Resize => help.get_keybinds_for_mode(InputMode::Resize),
        InputMode::Move => help.get_keybinds_for_mode(InputMode::Move),
        InputMode::Scroll => help.get_keybinds_for_mode(InputMode::Scroll),
        InputMode::Search => help.get_keybinds_for_mode(InputMode::Search),
        InputMode::Session => help.get_keybinds_for_mode(InputMode::Session),
        _ => help.get_mode_keybinds(),
    };
    let mut parts = vec![];

    match help.mode {
        InputMode::Normal => {
            // Show mode-switching keys from normal mode with more descriptive labels
            let keybindings = [
                (Action::SwitchToMode(InputMode::Pane), "pane"),
                (Action::SwitchToMode(InputMode::Tab), "tab"),
                (Action::SwitchToMode(InputMode::Resize), "resize"),
                (Action::SwitchToMode(InputMode::Move), "move"),
                (Action::SwitchToMode(InputMode::Scroll), "scroll"),
                (Action::SwitchToMode(InputMode::Search), "search"),
                (Action::SwitchToMode(InputMode::Session), "session"),
                (Action::Quit, "quit"),
            ];

            for (action, label) in keybindings {
                let keys = action_key(&keymap, &[action]);
                if !keys.is_empty() {
                    let styled_keys = style_key_with_modifier(&keys, &help.style.colors, None);
                    parts.extend(styled_keys);
                    parts.push(Style::new().paint(format!(" {} ", label)));
                }
            }
        }
        InputMode::Pane => {
            // Use the original implementation pattern with single_action_key and TO_NORMAL
            let pane_actions = [
                (&[Action::NewPane(None, None, false), TO_NORMAL][..], "new"),
                (&[Action::CloseFocus, TO_NORMAL][..], "close"),
                (
                    &[Action::ToggleFocusFullscreen, TO_NORMAL][..],
                    "fullscreen",
                ),
                (&[Action::ToggleFloatingPanes, TO_NORMAL][..], "floating"),
                (&[Action::TogglePaneEmbedOrFloating, TO_NORMAL][..], "embed"),
                (
                    &[
                        Action::NewPane(Some(Direction::Right), None, false),
                        TO_NORMAL,
                    ][..],
                    "split right",
                ),
                (
                    &[
                        Action::NewPane(Some(Direction::Down), None, false),
                        TO_NORMAL,
                    ][..],
                    "split down",
                ),
            ];

            for (actions, label) in pane_actions {
                let keys = single_action_key(&keymap, actions);
                if !keys.is_empty() {
                    let styled_keys = style_key_with_modifier(&keys, &help.style.colors, None);
                    parts.extend(styled_keys);
                    parts.push(Style::new().paint(format!(" {} ", label)));
                }
            }

            // Rename uses a different pattern (SwitchToMode + PaneNameInput)
            let rename_keys = single_action_key(
                &keymap,
                &[
                    Action::SwitchToMode(InputMode::RenamePane),
                    Action::PaneNameInput(vec![0]),
                ],
            );
            if !rename_keys.is_empty() {
                let styled_keys = style_key_with_modifier(&rename_keys, &help.style.colors, None);
                parts.extend(styled_keys);
                let styled_desc = style_description("rename", &help.style.colors);
                parts.extend(styled_desc);
            }

            // Focus movement keys (hjkl or arrow keys)
            let focus_keys = action_key_group(
                &keymap,
                &[
                    &[Action::MoveFocus(Direction::Left)],
                    &[Action::MoveFocus(Direction::Down)],
                    &[Action::MoveFocus(Direction::Up)],
                    &[Action::MoveFocus(Direction::Right)],
                ],
            );
            if !focus_keys.is_empty() {
                let styled_keys = style_key_with_modifier(&focus_keys, &help.style.colors, None);
                parts.extend(styled_keys);
                let styled_desc = style_description("move", &help.style.colors);
                parts.extend(styled_desc);
            }

            // Select pane (return to normal) - find preferred Enter key
            let to_normal_keys = action_key(&keymap, &[TO_NORMAL]);
            let select_key = if to_normal_keys.contains(&KeyWithModifier::new(BareKey::Enter)) {
                vec![KeyWithModifier::new(BareKey::Enter)]
            } else {
                to_normal_keys.into_iter().take(1).collect()
            };
            if !select_key.is_empty() {
                let styled_keys = style_key_with_modifier(&select_key, &help.style.colors, None);
                parts.extend(styled_keys);
                let styled_desc = style_description("select", &help.style.colors);
                parts.extend(styled_desc);
            }
        }
        InputMode::Tab => {
            // Use original implementation pattern for tab operations
            let tab_actions = [
                (
                    &[
                        Action::NewTab(None, vec![], None, None, None, true),
                        TO_NORMAL,
                    ][..],
                    "new",
                ),
                (&[Action::CloseTab, TO_NORMAL][..], "close"),
                (&[Action::BreakPane, TO_NORMAL][..], "break pane"),
                (&[Action::ToggleActiveSyncTab, TO_NORMAL][..], "sync"),
            ];

            for (actions, label) in tab_actions {
                let keys = single_action_key(&keymap, actions);
                if !keys.is_empty() {
                    let styled_keys = style_key_with_modifier(&keys, &help.style.colors, None);
                    parts.extend(styled_keys);
                    parts.push(Style::new().paint(format!(" {} ", label)));
                }
            }

            // Rename uses different pattern
            let rename_keys = single_action_key(
                &keymap,
                &[
                    Action::SwitchToMode(InputMode::RenameTab),
                    Action::TabNameInput(vec![0]),
                ],
            );
            if !rename_keys.is_empty() {
                let styled_keys = style_key_with_modifier(&rename_keys, &help.style.colors, None);
                parts.extend(styled_keys);
                let styled_desc = style_description("rename", &help.style.colors);
                parts.extend(styled_desc);
            }

            // Focus movement - special handling like original
            let focus_keys_full = action_key_group(
                &keymap,
                &[&[Action::GoToPreviousTab], &[Action::GoToNextTab]],
            );
            let focus_keys = if focus_keys_full.contains(&KeyWithModifier::new(BareKey::Left))
                && focus_keys_full.contains(&KeyWithModifier::new(BareKey::Right))
            {
                vec![
                    KeyWithModifier::new(BareKey::Left),
                    KeyWithModifier::new(BareKey::Right),
                ]
            } else {
                focus_keys_full
            };
            if !focus_keys.is_empty() {
                let styled_keys = style_key_with_modifier(&focus_keys, &help.style.colors, None);
                parts.extend(styled_keys);
                let styled_desc = style_description("move", &help.style.colors);
                parts.extend(styled_desc);
            }

            // Select pane (return to normal)
            let to_normal_keys = action_key(&keymap, &[TO_NORMAL]);
            let select_key = if to_normal_keys.contains(&KeyWithModifier::new(BareKey::Enter)) {
                vec![KeyWithModifier::new(BareKey::Enter)]
            } else {
                to_normal_keys.into_iter().take(1).collect()
            };
            if !select_key.is_empty() {
                let styled_keys = style_key_with_modifier(&select_key, &help.style.colors, None);
                parts.extend(styled_keys);
                let styled_desc = style_description("select", &help.style.colors);
                parts.extend(styled_desc);
            }
        }
        InputMode::Resize => {
            // Use original implementation pattern for resize operations
            let resize_keys = action_key_group(
                &keymap,
                &[
                    &[Action::Resize(Resize::Increase, None)],
                    &[Action::Resize(Resize::Decrease, None)],
                ],
            );
            if !resize_keys.is_empty() {
                let styled_keys = style_key_with_modifier(&resize_keys, &help.style.colors, None);
                parts.extend(styled_keys);
                let styled_desc = style_description("resize", &help.style.colors);
                parts.extend(styled_desc);
            }

            // Directional resize increase
            let resize_increase_keys = action_key_group(
                &keymap,
                &[
                    &[Action::Resize(Resize::Increase, Some(Direction::Left))],
                    &[Action::Resize(Resize::Increase, Some(Direction::Down))],
                    &[Action::Resize(Resize::Increase, Some(Direction::Up))],
                    &[Action::Resize(Resize::Increase, Some(Direction::Right))],
                ],
            );
            if !resize_increase_keys.is_empty() {
                let styled_keys =
                    style_key_with_modifier(&resize_increase_keys, &help.style.colors, None);
                parts.extend(styled_keys);
                let styled_desc = style_description("increase", &help.style.colors);
                parts.extend(styled_desc);
            }

            // Directional resize decrease
            let resize_decrease_keys = action_key_group(
                &keymap,
                &[
                    &[Action::Resize(Resize::Decrease, Some(Direction::Left))],
                    &[Action::Resize(Resize::Decrease, Some(Direction::Down))],
                    &[Action::Resize(Resize::Decrease, Some(Direction::Up))],
                    &[Action::Resize(Resize::Decrease, Some(Direction::Right))],
                ],
            );
            if !resize_decrease_keys.is_empty() {
                let styled_keys =
                    style_key_with_modifier(&resize_decrease_keys, &help.style.colors, None);
                parts.extend(styled_keys);
                let styled_desc = style_description("decrease", &help.style.colors);
                parts.extend(styled_desc);
            }

            // Select pane (return to normal)
            let to_normal_keys = action_key(&keymap, &[TO_NORMAL]);
            let select_key = if to_normal_keys.contains(&KeyWithModifier::new(BareKey::Enter)) {
                vec![KeyWithModifier::new(BareKey::Enter)]
            } else {
                to_normal_keys.into_iter().take(1).collect()
            };
            if !select_key.is_empty() {
                let styled_keys = style_key_with_modifier(&select_key, &help.style.colors, None);
                parts.extend(styled_keys);
                let styled_desc = style_description("select", &help.style.colors);
                parts.extend(styled_desc);
            }
        }
        InputMode::Move => {
            // Use original implementation pattern for move operations
            let move_keys = action_key_group(
                &keymap,
                &[
                    &[Action::MovePane(Some(Direction::Left))],
                    &[Action::MovePane(Some(Direction::Down))],
                    &[Action::MovePane(Some(Direction::Up))],
                    &[Action::MovePane(Some(Direction::Right))],
                ],
            );
            if !move_keys.is_empty() {
                let styled_keys = style_key_with_modifier(&move_keys, &help.style.colors, None);
                parts.extend(styled_keys);
                let styled_desc = style_description("move", &help.style.colors);
                parts.extend(styled_desc);
            }

            // Select pane (return to normal)
            let to_normal_keys = action_key(&keymap, &[TO_NORMAL]);
            let select_key = if to_normal_keys.contains(&KeyWithModifier::new(BareKey::Enter)) {
                vec![KeyWithModifier::new(BareKey::Enter)]
            } else {
                to_normal_keys.into_iter().take(1).collect()
            };
            if !select_key.is_empty() {
                let styled_keys = style_key_with_modifier(&select_key, &help.style.colors, None);
                parts.extend(styled_keys);
                let styled_desc = style_description("select", &help.style.colors);
                parts.extend(styled_desc);
            }
        }
        InputMode::Scroll => {
            // Use original implementation pattern for scroll operations
            // Enter search term
            let search_keys = action_key(
                &keymap,
                &[
                    Action::SwitchToMode(InputMode::EnterSearch),
                    Action::SearchInput(vec![0]),
                ],
            );
            if !search_keys.is_empty() {
                let styled_keys = style_key_with_modifier(&search_keys, &help.style.colors, None);
                parts.extend(styled_keys);
                let styled_desc = style_description("search", &help.style.colors);
                parts.extend(styled_desc);
            }

            // Scroll operations
            let scroll_keys =
                action_key_group(&keymap, &[&[Action::ScrollDown], &[Action::ScrollUp]]);
            if !scroll_keys.is_empty() {
                let styled_keys = style_key_with_modifier(&scroll_keys, &help.style.colors, None);
                parts.extend(styled_keys);
                let styled_desc = style_description("scroll", &help.style.colors);
                parts.extend(styled_desc);
            }

            // Page scroll operations
            let page_scroll_keys = action_key_group(
                &keymap,
                &[&[Action::PageScrollDown], &[Action::PageScrollUp]],
            );
            if !page_scroll_keys.is_empty() {
                let styled_keys =
                    style_key_with_modifier(&page_scroll_keys, &help.style.colors, None);
                parts.extend(styled_keys);
                let styled_desc = style_description("page", &help.style.colors);
                parts.extend(styled_desc);
            }

            // Half page scroll operations
            let half_page_scroll_keys = action_key_group(
                &keymap,
                &[&[Action::HalfPageScrollDown], &[Action::HalfPageScrollUp]],
            );
            if !half_page_scroll_keys.is_empty() {
                let styled_keys =
                    style_key_with_modifier(&half_page_scroll_keys, &help.style.colors, None);
                parts.extend(styled_keys);
                let styled_desc = style_description("half page", &help.style.colors);
                parts.extend(styled_desc);
            }

            // Edit scrollback
            let edit_keys = single_action_key(&keymap, &[Action::EditScrollback, TO_NORMAL]);
            if !edit_keys.is_empty() {
                let styled_keys = style_key_with_modifier(&edit_keys, &help.style.colors, None);
                parts.extend(styled_keys);
                let styled_desc = style_description("edit", &help.style.colors);
                parts.extend(styled_desc);
            }

            // Select pane (return to normal)
            let to_normal_keys = action_key(&keymap, &[TO_NORMAL]);
            let select_key = if to_normal_keys.contains(&KeyWithModifier::new(BareKey::Enter)) {
                vec![KeyWithModifier::new(BareKey::Enter)]
            } else {
                to_normal_keys.into_iter().take(1).collect()
            };
            if !select_key.is_empty() {
                let styled_keys = style_key_with_modifier(&select_key, &help.style.colors, None);
                parts.extend(styled_keys);
                let styled_desc = style_description("select", &help.style.colors);
                parts.extend(styled_desc);
            }
        }
        InputMode::Search => {
            // Use original implementation pattern for search operations
            // Enter search term
            let search_keys = action_key(
                &keymap,
                &[
                    Action::SwitchToMode(InputMode::EnterSearch),
                    Action::SearchInput(vec![0]),
                ],
            );
            if !search_keys.is_empty() {
                let styled_keys = style_key_with_modifier(&search_keys, &help.style.colors, None);
                parts.extend(styled_keys);
                let styled_desc = style_description("search", &help.style.colors);
                parts.extend(styled_desc);
            }

            // Scroll operations (same as scroll mode)
            let scroll_keys =
                action_key_group(&keymap, &[&[Action::ScrollDown], &[Action::ScrollUp]]);
            if !scroll_keys.is_empty() {
                let styled_keys = style_key_with_modifier(&scroll_keys, &help.style.colors, None);
                parts.extend(styled_keys);
                let styled_desc = style_description("scroll", &help.style.colors);
                parts.extend(styled_desc);
            }

            // Page scroll operations
            let page_scroll_keys = action_key_group(
                &keymap,
                &[&[Action::PageScrollDown], &[Action::PageScrollUp]],
            );
            if !page_scroll_keys.is_empty() {
                let styled_keys =
                    style_key_with_modifier(&page_scroll_keys, &help.style.colors, None);
                parts.extend(styled_keys);
                let styled_desc = style_description("page", &help.style.colors);
                parts.extend(styled_desc);
            }

            // Half page scroll operations
            let half_page_scroll_keys = action_key_group(
                &keymap,
                &[&[Action::HalfPageScrollDown], &[Action::HalfPageScrollUp]],
            );
            if !half_page_scroll_keys.is_empty() {
                let styled_keys =
                    style_key_with_modifier(&half_page_scroll_keys, &help.style.colors, None);
                parts.extend(styled_keys);
                let styled_desc = style_description("half page", &help.style.colors);
                parts.extend(styled_desc);
            }

            // Search directions
            let search_down_keys = action_key(&keymap, &[Action::Search(SearchDirection::Down)]);
            if !search_down_keys.is_empty() {
                let styled_keys =
                    style_key_with_modifier(&search_down_keys, &help.style.colors, None);
                parts.extend(styled_keys);
                let styled_desc = style_description("down", &help.style.colors);
                parts.extend(styled_desc);
            }

            let search_up_keys = action_key(&keymap, &[Action::Search(SearchDirection::Up)]);
            if !search_up_keys.is_empty() {
                let styled_keys =
                    style_key_with_modifier(&search_up_keys, &help.style.colors, None);
                parts.extend(styled_keys);
                let styled_desc = style_description("up", &help.style.colors);
                parts.extend(styled_desc);
            }

            // Select pane (return to normal)
            let to_normal_keys = action_key(&keymap, &[TO_NORMAL]);
            let select_key = if to_normal_keys.contains(&KeyWithModifier::new(BareKey::Enter)) {
                vec![KeyWithModifier::new(BareKey::Enter)]
            } else {
                to_normal_keys.into_iter().take(1).collect()
            };
            if !select_key.is_empty() {
                let styled_keys = style_key_with_modifier(&select_key, &help.style.colors, None);
                parts.extend(styled_keys);
                let styled_desc = style_description("select", &help.style.colors);
                parts.extend(styled_desc);
            }
        }
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
                    parts.push(Style::new().paint(format!(" {} ", label)));
                }
            }
        }
        _ => {
            // For other modes, just show how to get back to normal
            let keys = action_key(&keymap, &[Action::SwitchToMode(InputMode::Normal)]);
            if !keys.is_empty() {
                let styled_keys = style_key_with_modifier(&keys, &help.style.colors, None);
                parts.extend(styled_keys);
                let styled_desc = style_description("normal", &help.style.colors);
                parts.extend(styled_desc);
            }
        }
    }

    if parts.is_empty() {
        return LinePart {
            part: String::new(),
            len: 0,
        };
    }

    // Build the final string and calculate length
    use ansi_term::{unstyled_len, ANSIStrings};
    let ansi_strings = ANSIStrings(&parts);
    let formatted = format!(" {}", ansi_strings);
    let len = 1 + unstyled_len(&ansi_strings);

    if len <= max_len {
        LinePart {
            part: formatted,
            len,
        }
    } else {
        LinePart {
            part: String::new(),
            len: 0,
        }
    }
}
