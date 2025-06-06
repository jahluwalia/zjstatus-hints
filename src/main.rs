mod tip;
mod ui;

use ansi_term::{
    ANSIString,
    Colour::{Fixed, RGB},
    Style,
};
use std::collections::BTreeMap;
use std::fmt::{Display, Error, Formatter};
use zellij_tile::prelude::actions::Action;
use zellij_tile::prelude::*;
use zellij_tile_utils::palette_match;

use tip::utils::get_cached_tip_name;
use ui::one_line_ui;

// for more of these, copy paste from: https://en.wikipedia.org/wiki/Box-drawing_character
static ARROW_SEPARATOR: &str = "";

#[derive(Default)]
struct State {
    tabs: Vec<TabInfo>,
    tip_name: String,
    mode_info: ModeInfo,
    text_copy_destination: Option<CopyDestination>,
    display_system_clipboard_failure: bool,
    classic_ui: bool,
    base_mode_is_locked: bool,
    max_length: usize,
    overflow_str: String,
}

register_plugin!(State);

#[derive(Default)]
pub struct LinePart {
    part: String,
    len: usize,
}

impl LinePart {
    pub fn append(&mut self, to_append: &LinePart) {
        self.part.push_str(&to_append.part);
        self.len += to_append.len;
    }
}

impl Display for LinePart {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.part)
    }
}

/// Shorthand for `Action::SwitchToMode(InputMode::Normal)`.
const TO_NORMAL: Action = Action::SwitchToMode(InputMode::Normal);

#[derive(Clone, Copy)]
pub struct ColoredElements {
    pub selected: SegmentStyle,
    pub unselected: SegmentStyle,
    pub unselected_alternate: SegmentStyle,
    pub disabled: SegmentStyle,
    // superkey
    pub superkey_prefix: Style,
    pub superkey_suffix_separator: Style,
}

#[derive(Clone, Copy)]
pub struct SegmentStyle {
    pub prefix_separator: Style,
    pub char_left_separator: Style,
    pub char_shortcut: Style,
    pub char_right_separator: Style,
    pub styled_text: Style,
    pub suffix_separator: Style,
}

pub fn get_common_modifiers(mut keyvec: Vec<&KeyWithModifier>) -> Vec<KeyModifier> {
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
        self.tip_name = get_cached_tip_name();
        self.classic_ui = configuration
            .get("classic")
            .map(|c| c == "true")
            .unwrap_or(false);
        self.max_length = configuration
            .get("max_length")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        self.overflow_str = configuration
            .get("overflow_str")
            .cloned()
            .unwrap_or_else(|| "...".to_string());

        // TODO: The user can't approve/deny permissions because they can't select the pane, I think we need to open a popup or something
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::FullHdAccess,
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
        match event {
            Event::ModeUpdate(mode_info) => {
                if self.mode_info != mode_info {
                    should_render = true;
                }
                self.mode_info = mode_info;
                self.base_mode_is_locked = self.mode_info.base_mode == Some(InputMode::Locked);
            }
            Event::TabUpdate(tabs) => {
                if self.tabs != tabs {
                    should_render = true;
                }
                self.tabs = tabs;
            }
            Event::CopyToClipboard(copy_destination) => {
                match self.text_copy_destination {
                    Some(text_copy_destination) => {
                        if text_copy_destination != copy_destination {
                            should_render = true;
                        }
                    }
                    None => {
                        should_render = true;
                    }
                }
                self.text_copy_destination = Some(copy_destination);
            }
            Event::SystemClipboardFailure => {
                should_render = true;
                self.display_system_clipboard_failure = true;
            }
            Event::InputReceived => {
                if self.text_copy_destination.is_some() || self.display_system_clipboard_failure {
                    should_render = true;
                }
                self.text_copy_destination = None;
                self.display_system_clipboard_failure = false;
            }
            _ => {}
        };
        should_render
    }

    fn render(&mut self, _rows: usize, cols: usize) {
        let supports_arrow_fonts = !self.mode_info.capabilities.arrow_fonts;
        let separator = if supports_arrow_fonts {
            ARROW_SEPARATOR
        } else {
            ""
        };

        let background = self.mode_info.style.colors.text_unselected.background;

        // Always use single line UI
        let fill_bg = match background {
            PaletteColor::Rgb((r, g, b)) => format!("\u{1b}[48;2;{};{};{}m\u{1b}[0K", r, g, b),
            PaletteColor::EightBit(color) => format!("\u{1b}[48;5;{}m\u{1b}[0K", color),
        };
        let active_tab = self.tabs.iter().find(|t| t.active);

        let output = format!(
            "{}{}",
            one_line_ui(
                &self.mode_info,
                active_tab,
                cols,
                separator,
                self.base_mode_is_locked,
                self.text_copy_destination,
                self.display_system_clipboard_failure,
                &self.tip_name,
            ),
            fill_bg,
        );

        // Send output to both zjstatus plugin and stdout
        self.send_to_zjstatus(&output);
        print!("{}", output);
    }
}

impl State {
    fn send_to_zjstatus(&self, message: &str) {
        if !message.is_empty() {
            let visible_len = self.calculate_visible_length(message);
            let output = if self.max_length > 0 && visible_len > self.max_length {
                self.truncate_ansi_string(message, self.max_length)
            } else {
                message.to_string()
            };
            pipe_message_to_plugin(
                MessageToPlugin::new("pipe")
                    .with_payload(format!("zjstatus::pipe::pipe_zjstatus_hints::{}", output)),
            );
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

// Helper functions for tip system
pub fn action_key(
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

pub fn action_key_group(
    keymap: &[(KeyWithModifier, Vec<Action>)],
    actions: &[&[Action]],
) -> Vec<KeyWithModifier> {
    let mut ret = vec![];
    for action in actions {
        ret.extend(action_key(keymap, action));
    }
    ret
}

pub fn single_action_key(
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

/// Style a keybinding hint with saturated background for keys and less saturated for description
pub fn style_key_with_modifier(
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

    // Create key display without brackets
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

    // Special handling of some pre-defined keygroups
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

    // Add space before keybinding segment
    ret.push(Style::new().paint(" "));

    // Add modifier if present with plus
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

    // Add keys without brackets
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

    // Close keybinding segment with space
    ret.push(Style::new().fg(contrasting_fg).on(saturated_bg).paint(" "));

    ret
}

/// Style a description with less saturated background
pub fn style_description(description: &str, palette: &Styling) -> Vec<ANSIString<'static>> {
    let less_saturated_bg = palette_match!(palette.text_unselected.background);
    let contrasting_fg = palette_match!(palette.text_unselected.base);

    vec![Style::new()
        .fg(contrasting_fg)
        .on(less_saturated_bg)
        .paint(format!(" {} ", description))]
}
