use ansi_term::{
    ANSIString,
    Colour::{Fixed, RGB},
    Style,
};
use std::collections::BTreeMap;
use zellij_tile::prelude::actions::Action;
use zellij_tile::prelude::actions::SearchDirection;
use zellij_tile::prelude::*;
use zellij_tile_utils::palette_match;

#[derive(Default)]
struct State {
    pipe_name: String,
    mode_info: ModeInfo,
    base_mode_is_locked: bool,
    max_length: usize,
    overflow_str: String,
}

register_plugin!(State);

const TO_NORMAL: Action = Action::SwitchToMode(InputMode::Normal);

fn get_common_modifiers(mut keyvec: Vec<&KeyWithModifier>) -> Vec<KeyModifier> {
    if keyvec.is_empty() {
        return vec![];
    }
    let mut common_modifiers = keyvec.pop().unwrap().key_modifiers.clone();
    for key in keyvec {
        common_modifiers = common_modifiers
            .intersection(&key.key_modifiers)
            .cloned()
            .collect();
    }
    common_modifiers.into_iter().collect()
}

impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.max_length = configuration
            .get("max_length")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        self.overflow_str = configuration
            .get("overflow_str")
            .cloned()
            .unwrap_or_else(|| "...".to_string());
        self.pipe_name = configuration
            .get("pipe_name")
            .cloned()
            .unwrap_or_else(|| "zjstatus_hints".to_string());

        // TODO: The user can't approve/deny permissions because they can't select the pane, I think we need to open a popup or something
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::MessageAndLaunchOtherPlugins,
        ]);

        set_selectable(false);
        subscribe(&[
            EventType::ModeUpdate,
            EventType::TabUpdate,
            EventType::CopyToClipboard,
            EventType::InputReceived,
            EventType::SystemClipboardFailure,
        ]);
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;
        if let Event::ModeUpdate(mode_info) = event {
            if self.mode_info != mode_info {
                should_render = true;
            }
            self.mode_info = mode_info;
            self.base_mode_is_locked = self.mode_info.base_mode == Some(InputMode::Locked);
        };
        should_render
    }

    fn render(&mut self, _rows: usize, cols: usize) {
        let background = self.mode_info.style.colors.text_unselected.background;
        let fill_bg = match background {
            PaletteColor::Rgb((r, g, b)) => format!("\u{1b}[48;2;{};{};{}m\u{1b}[0K", r, g, b),
            PaletteColor::EightBit(color) => format!("\u{1b}[48;5;{}m\u{1b}[0K", color),
        };
        if self.mode_info.mode == InputMode::Locked && self.base_mode_is_locked {
            self.display(&fill_bg);
        } else {
            self.display(&format!(
                "{}{}",
                render_hints(&self.mode_info, cols),
                fill_bg
            ));
        }
    }
}

impl State {
    fn display(&self, message: &str) {
        if !message.is_empty() {
            let visible_len = self.calculate_visible_length(message);
            let output = if self.max_length > 0 && visible_len > self.max_length {
                self.truncate_ansi_string(message, self.max_length)
            } else {
                message.to_string()
            };
            pipe_message_to_plugin(MessageToPlugin::new("pipe").with_payload(format!(
                "zjstatus::pipe::pipe_{}::{}",
                self.pipe_name, output
            )));
            print!("{}", output);
        }
    }

    fn truncate_ansi_string(&self, text: &str, max_len: usize) -> String {
        let visible_len = self.calculate_visible_length(text);
        let overflow_len = self.overflow_str.len();

        if visible_len <= max_len {
            return text.to_string();
        }

        if max_len <= overflow_len {
            return self.overflow_str.clone();
        }

        let target_len = max_len - overflow_len;
        let mut result = String::new();
        let mut current_len = 0;
        let chars = text.chars();
        let mut in_escape = false;

        for ch in chars {
            if ch == '\x1b' {
                in_escape = true;
                result.push(ch);
            } else if in_escape {
                result.push(ch);
                if ch == 'm' {
                    in_escape = false;
                }
            } else {
                if current_len >= target_len {
                    break;
                }
                result.push(ch);
                current_len += 1;
            }
        }

        result.push_str(&self.overflow_str);
        result
    }

    fn calculate_visible_length(&self, text: &str) -> usize {
        let mut len = 0;
        let chars = text.chars();
        let mut in_escape = false;

        for ch in chars {
            if ch == '\x1b' {
                in_escape = true;
            } else if in_escape {
                if ch == 'm' {
                    in_escape = false;
                }
            } else {
                len += 1;
            }
        }

        len
    }
}

fn action_key(
    keymap: &[(KeyWithModifier, Vec<Action>)],
    action: &[Action],
) -> Vec<KeyWithModifier> {
    keymap
        .iter()
        .filter_map(|(key, acvec)| {
            let matching = acvec
                .iter()
                .zip(action)
                .filter(|(a, b)| a.shallow_eq(b))
                .count();

            if matching == acvec.len() && matching == action.len() {
                Some(key.clone())
            } else {
                None
            }
        })
        .collect()
}

fn action_key_group(
    keymap: &[(KeyWithModifier, Vec<Action>)],
    actions: &[&[Action]],
) -> Vec<KeyWithModifier> {
    let mut ret = vec![];
    for action in actions {
        ret.extend(action_key(keymap, action));
    }
    ret
}

fn single_action_key(
    keymap: &[(KeyWithModifier, Vec<Action>)],
    action: &[Action],
) -> Vec<KeyWithModifier> {
    let mut matching = keymap.iter().find_map(|(key, acvec)| {
        if acvec.iter().next() == action.iter().next() {
            Some(key.clone())
        } else {
            None
        }
    });
    if let Some(matching) = matching.take() {
        vec![matching]
    } else {
        vec![]
    }
}

fn style_key_with_modifier(
    keyvec: &[KeyWithModifier],
    palette: &Styling,
) -> Vec<ANSIString<'static>> {
    if keyvec.is_empty() {
        return vec![];
    }

    let saturated_bg = palette_match!(palette.ribbon_unselected.background);
    let contrasting_fg = palette_match!(palette.ribbon_unselected.base);
    let mut ret = vec![];

    let common_modifiers = get_common_modifiers(keyvec.iter().collect());

    let modifier_str = common_modifiers
        .iter()
        .map(|m| m.to_string())
        .collect::<Vec<_>>()
        .join("-");

    let key_display = keyvec
        .iter()
        .map(|key| {
            if common_modifiers.is_empty() {
                format!("{}", key)
            } else {
                let key_modifier_for_key = key
                    .key_modifiers
                    .iter()
                    .filter(|m| !common_modifiers.contains(m))
                    .map(|m| m.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                if key_modifier_for_key.is_empty() {
                    format!("{}", key.bare_key)
                } else {
                    format!("{} {}", key_modifier_for_key, key.bare_key)
                }
            }
        })
        .collect::<Vec<String>>();

    let key_string = key_display.join("");
    let key_separator = match &key_string[..] {
        "HJKL" => "",
        "hjkl" => "",
        "←↓↑→" => "",
        "←→" => "",
        "↓↑" => "",
        "[]" => "",
        _ => "|",
    };

    ret.push(Style::new().paint(" "));

    if !modifier_str.is_empty() {
        ret.push(
            Style::new()
                .fg(contrasting_fg)
                .on(saturated_bg)
                .bold()
                .paint(format!(" {} + ", modifier_str)),
        );
    } else {
        ret.push(Style::new().fg(contrasting_fg).on(saturated_bg).paint(" "));
    }

    for (idx, key) in key_display.iter().enumerate() {
        if idx > 0 && !key_separator.is_empty() {
            ret.push(
                Style::new()
                    .fg(contrasting_fg)
                    .on(saturated_bg)
                    .paint(key_separator),
            );
        }
        ret.push(
            Style::new()
                .fg(contrasting_fg)
                .on(saturated_bg)
                .bold()
                .paint(key.clone()),
        );
    }

    ret.push(Style::new().fg(contrasting_fg).on(saturated_bg).paint(" "));

    ret
}

fn style_description(description: &str, palette: &Styling) -> Vec<ANSIString<'static>> {
    let less_saturated_bg = palette_match!(palette.text_unselected.background);
    let contrasting_fg = palette_match!(palette.text_unselected.base);

    vec![Style::new()
        .fg(contrasting_fg)
        .on(less_saturated_bg)
        .paint(format!(" {} ", description))]
}

fn session_manager_key(keymap: &[(KeyWithModifier, Vec<Action>)]) -> Vec<KeyWithModifier> {
    let mut matching = keymap.iter().find_map(|(key, acvec)| {
        let has_match = acvec.iter().any(|a| a.launches_plugin("session-manager"));
        if has_match {
            Some(key.clone())
        } else {
            None
        }
    });
    if let Some(matching) = matching.take() {
        vec![matching]
    } else {
        vec![]
    }
}

fn plugin_manager_key(keymap: &[(KeyWithModifier, Vec<Action>)]) -> Vec<KeyWithModifier> {
    let mut matching = keymap.iter().find_map(|(key, acvec)| {
        let has_match = acvec.iter().any(|a| a.launches_plugin("plugin-manager"));
        if has_match {
            Some(key.clone())
        } else {
            None
        }
    });
    if let Some(matching) = matching.take() {
        vec![matching]
    } else {
        vec![]
    }
}

fn about_key(keymap: &[(KeyWithModifier, Vec<Action>)]) -> Vec<KeyWithModifier> {
    let mut matching = keymap.iter().find_map(|(key, acvec)| {
        let has_match = acvec.iter().any(|a| a.launches_plugin("zellij:about"));
        if has_match {
            Some(key.clone())
        } else {
            None
        }
    });
    if let Some(matching) = matching.take() {
        vec![matching]
    } else {
        vec![]
    }
}

fn configuration_key(keymap: &[(KeyWithModifier, Vec<Action>)]) -> Vec<KeyWithModifier> {
    let mut matching = keymap.iter().find_map(|(key, acvec)| {
        let has_match = acvec.iter().any(|a| a.launches_plugin("configuration"));
        if has_match {
            Some(key.clone())
        } else {
            None
        }
    });
    if let Some(matching) = matching.take() {
        vec![matching]
    } else {
        vec![]
    }
}

fn render_hints(help: &ModeInfo, max_len: usize) -> String {
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
                    let styled_keys = style_key_with_modifier(&keys, &help.style.colors);
                    parts.extend(styled_keys);
                    parts.push(Style::new().paint(format!(" {} ", label)));
                }
            }
        }
        InputMode::Pane => {
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
                    let styled_keys = style_key_with_modifier(&keys, &help.style.colors);
                    parts.extend(styled_keys);
                    parts.push(Style::new().paint(format!(" {} ", label)));
                }
            }

            let rename_keys = single_action_key(
                &keymap,
                &[
                    Action::SwitchToMode(InputMode::RenamePane),
                    Action::PaneNameInput(vec![0]),
                ],
            );
            if !rename_keys.is_empty() {
                let styled_keys = style_key_with_modifier(&rename_keys, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("rename", &help.style.colors);
                parts.extend(styled_desc);
            }

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
                let styled_keys = style_key_with_modifier(&focus_keys, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("move", &help.style.colors);
                parts.extend(styled_desc);
            }

            let to_normal_keys = action_key(&keymap, &[TO_NORMAL]);
            let select_key = if to_normal_keys.contains(&KeyWithModifier::new(BareKey::Enter)) {
                vec![KeyWithModifier::new(BareKey::Enter)]
            } else {
                to_normal_keys.into_iter().take(1).collect()
            };
            if !select_key.is_empty() {
                let styled_keys = style_key_with_modifier(&select_key, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("select", &help.style.colors);
                parts.extend(styled_desc);
            }
        }
        InputMode::Tab => {
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
                    let styled_keys = style_key_with_modifier(&keys, &help.style.colors);
                    parts.extend(styled_keys);
                    parts.push(Style::new().paint(format!(" {} ", label)));
                }
            }

            let rename_keys = single_action_key(
                &keymap,
                &[
                    Action::SwitchToMode(InputMode::RenameTab),
                    Action::TabNameInput(vec![0]),
                ],
            );
            if !rename_keys.is_empty() {
                let styled_keys = style_key_with_modifier(&rename_keys, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("rename", &help.style.colors);
                parts.extend(styled_desc);
            }

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
                let styled_keys = style_key_with_modifier(&focus_keys, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("move", &help.style.colors);
                parts.extend(styled_desc);
            }

            let to_normal_keys = action_key(&keymap, &[TO_NORMAL]);
            let select_key = if to_normal_keys.contains(&KeyWithModifier::new(BareKey::Enter)) {
                vec![KeyWithModifier::new(BareKey::Enter)]
            } else {
                to_normal_keys.into_iter().take(1).collect()
            };
            if !select_key.is_empty() {
                let styled_keys = style_key_with_modifier(&select_key, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("select", &help.style.colors);
                parts.extend(styled_desc);
            }
        }
        InputMode::Resize => {
            let resize_keys = action_key_group(
                &keymap,
                &[
                    &[Action::Resize(Resize::Increase, None)],
                    &[Action::Resize(Resize::Decrease, None)],
                ],
            );
            if !resize_keys.is_empty() {
                let styled_keys = style_key_with_modifier(&resize_keys, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("resize", &help.style.colors);
                parts.extend(styled_desc);
            }

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
                    style_key_with_modifier(&resize_increase_keys, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("increase", &help.style.colors);
                parts.extend(styled_desc);
            }

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
                    style_key_with_modifier(&resize_decrease_keys, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("decrease", &help.style.colors);
                parts.extend(styled_desc);
            }

            let to_normal_keys = action_key(&keymap, &[TO_NORMAL]);
            let select_key = if to_normal_keys.contains(&KeyWithModifier::new(BareKey::Enter)) {
                vec![KeyWithModifier::new(BareKey::Enter)]
            } else {
                to_normal_keys.into_iter().take(1).collect()
            };
            if !select_key.is_empty() {
                let styled_keys = style_key_with_modifier(&select_key, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("select", &help.style.colors);
                parts.extend(styled_desc);
            }
        }
        InputMode::Move => {
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
                let styled_keys = style_key_with_modifier(&move_keys, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("move", &help.style.colors);
                parts.extend(styled_desc);
            }

            let to_normal_keys = action_key(&keymap, &[TO_NORMAL]);
            let select_key = if to_normal_keys.contains(&KeyWithModifier::new(BareKey::Enter)) {
                vec![KeyWithModifier::new(BareKey::Enter)]
            } else {
                to_normal_keys.into_iter().take(1).collect()
            };
            if !select_key.is_empty() {
                let styled_keys = style_key_with_modifier(&select_key, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("select", &help.style.colors);
                parts.extend(styled_desc);
            }
        }
        InputMode::Scroll => {
            let search_keys = action_key(
                &keymap,
                &[
                    Action::SwitchToMode(InputMode::EnterSearch),
                    Action::SearchInput(vec![0]),
                ],
            );
            if !search_keys.is_empty() {
                let styled_keys = style_key_with_modifier(&search_keys, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("search", &help.style.colors);
                parts.extend(styled_desc);
            }

            let scroll_keys =
                action_key_group(&keymap, &[&[Action::ScrollDown], &[Action::ScrollUp]]);
            if !scroll_keys.is_empty() {
                let styled_keys = style_key_with_modifier(&scroll_keys, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("scroll", &help.style.colors);
                parts.extend(styled_desc);
            }

            let page_scroll_keys = action_key_group(
                &keymap,
                &[&[Action::PageScrollDown], &[Action::PageScrollUp]],
            );
            if !page_scroll_keys.is_empty() {
                let styled_keys = style_key_with_modifier(&page_scroll_keys, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("page", &help.style.colors);
                parts.extend(styled_desc);
            }

            let half_page_scroll_keys = action_key_group(
                &keymap,
                &[&[Action::HalfPageScrollDown], &[Action::HalfPageScrollUp]],
            );
            if !half_page_scroll_keys.is_empty() {
                let styled_keys =
                    style_key_with_modifier(&half_page_scroll_keys, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("half page", &help.style.colors);
                parts.extend(styled_desc);
            }

            let edit_keys = single_action_key(&keymap, &[Action::EditScrollback, TO_NORMAL]);
            if !edit_keys.is_empty() {
                let styled_keys = style_key_with_modifier(&edit_keys, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("edit", &help.style.colors);
                parts.extend(styled_desc);
            }

            let to_normal_keys = action_key(&keymap, &[TO_NORMAL]);
            let select_key = if to_normal_keys.contains(&KeyWithModifier::new(BareKey::Enter)) {
                vec![KeyWithModifier::new(BareKey::Enter)]
            } else {
                to_normal_keys.into_iter().take(1).collect()
            };
            if !select_key.is_empty() {
                let styled_keys = style_key_with_modifier(&select_key, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("select", &help.style.colors);
                parts.extend(styled_desc);
            }
        }
        InputMode::Search => {
            let search_keys = action_key(
                &keymap,
                &[
                    Action::SwitchToMode(InputMode::EnterSearch),
                    Action::SearchInput(vec![0]),
                ],
            );
            if !search_keys.is_empty() {
                let styled_keys = style_key_with_modifier(&search_keys, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("search", &help.style.colors);
                parts.extend(styled_desc);
            }

            let scroll_keys =
                action_key_group(&keymap, &[&[Action::ScrollDown], &[Action::ScrollUp]]);
            if !scroll_keys.is_empty() {
                let styled_keys = style_key_with_modifier(&scroll_keys, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("scroll", &help.style.colors);
                parts.extend(styled_desc);
            }

            let page_scroll_keys = action_key_group(
                &keymap,
                &[&[Action::PageScrollDown], &[Action::PageScrollUp]],
            );
            if !page_scroll_keys.is_empty() {
                let styled_keys = style_key_with_modifier(&page_scroll_keys, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("page", &help.style.colors);
                parts.extend(styled_desc);
            }

            let half_page_scroll_keys = action_key_group(
                &keymap,
                &[&[Action::HalfPageScrollDown], &[Action::HalfPageScrollUp]],
            );
            if !half_page_scroll_keys.is_empty() {
                let styled_keys =
                    style_key_with_modifier(&half_page_scroll_keys, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("half page", &help.style.colors);
                parts.extend(styled_desc);
            }

            let search_down_keys = action_key(&keymap, &[Action::Search(SearchDirection::Down)]);
            if !search_down_keys.is_empty() {
                let styled_keys = style_key_with_modifier(&search_down_keys, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("down", &help.style.colors);
                parts.extend(styled_desc);
            }

            let search_up_keys = action_key(&keymap, &[Action::Search(SearchDirection::Up)]);
            if !search_up_keys.is_empty() {
                let styled_keys = style_key_with_modifier(&search_up_keys, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("up", &help.style.colors);
                parts.extend(styled_desc);
            }

            let to_normal_keys = action_key(&keymap, &[TO_NORMAL]);
            let select_key = if to_normal_keys.contains(&KeyWithModifier::new(BareKey::Enter)) {
                vec![KeyWithModifier::new(BareKey::Enter)]
            } else {
                to_normal_keys.into_iter().take(1).collect()
            };
            if !select_key.is_empty() {
                let styled_keys = style_key_with_modifier(&select_key, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("select", &help.style.colors);
                parts.extend(styled_desc);
            }
        }
        InputMode::Session => {
            let detach_keys = action_key(&keymap, &[Action::Detach]);
            if !detach_keys.is_empty() {
                let styled_keys = style_key_with_modifier(&detach_keys, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("detach", &help.style.colors);
                parts.extend(styled_desc);
            }

            let manager_keys = session_manager_key(&keymap);
            if !manager_keys.is_empty() {
                let styled_keys = style_key_with_modifier(&manager_keys, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("manager", &help.style.colors);
                parts.extend(styled_desc);
            }

            let config_keys = configuration_key(&keymap);
            if !config_keys.is_empty() {
                let styled_keys = style_key_with_modifier(&config_keys, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("config", &help.style.colors);
                parts.extend(styled_desc);
            }

            let plugin_keys = plugin_manager_key(&keymap);
            if !plugin_keys.is_empty() {
                let styled_keys = style_key_with_modifier(&plugin_keys, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("plugins", &help.style.colors);
                parts.extend(styled_desc);
            }

            let about_keys = about_key(&keymap);
            if !about_keys.is_empty() {
                let styled_keys = style_key_with_modifier(&about_keys, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("about", &help.style.colors);
                parts.extend(styled_desc);
            }

            let base_mode = help.base_mode;
            let to_basemode_keys = base_mode
                .map(|b| action_key(&keymap, &[Action::SwitchToMode(b)]))
                .unwrap_or_else(|| action_key(&keymap, &[TO_NORMAL]));
            let select_key = if to_basemode_keys.contains(&KeyWithModifier::new(BareKey::Enter)) {
                vec![KeyWithModifier::new(BareKey::Enter)]
            } else {
                to_basemode_keys.into_iter().take(1).collect()
            };
            if !select_key.is_empty() {
                let styled_keys = style_key_with_modifier(&select_key, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("select", &help.style.colors);
                parts.extend(styled_desc);
            }
        }
        _ => {
            let keys = action_key(&keymap, &[Action::SwitchToMode(InputMode::Normal)]);
            if !keys.is_empty() {
                let styled_keys = style_key_with_modifier(&keys, &help.style.colors);
                parts.extend(styled_keys);
                let styled_desc = style_description("normal", &help.style.colors);
                parts.extend(styled_desc);
            }
        }
    }

    if parts.is_empty() {
        return String::new();
    }

    use ansi_term::{unstyled_len, ANSIStrings};
    let ansi_strings = ANSIStrings(&parts);
    let formatted = format!(" {}", ansi_strings);
    let len = 1 + unstyled_len(&ansi_strings);

    if len <= max_len {
        formatted
    } else {
        String::new()
    }
}
