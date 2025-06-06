mod ui;
mod tip;

use std::collections::BTreeMap;
use std::fmt::{Display, Error, Formatter};
use zellij_tile::prelude::*;
use zellij_tile::prelude::actions::Action;

use ui::one_line_ui;
use tip::utils::get_cached_tip_name;

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


impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.tip_name = get_cached_tip_name();
        self.classic_ui = configuration
            .get("classic")
            .map(|c| c == "true")
            .unwrap_or(false);

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
                if self.text_copy_destination.is_some()
                    || self.display_system_clipboard_failure == true
                {
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
            pipe_message_to_plugin(
                MessageToPlugin::new("pipe")
                    .with_payload(format!("zjstatus::pipe::pipe_zjstatus_hints::{}", message)),
            );
        }
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

pub fn style_key_with_modifier(
    keyvec: &[KeyWithModifier],
    _palette: &Styling,
    _background: Option<PaletteColor>,
) -> Vec<ansi_term::ANSIString<'static>> {
    use ansi_term::Style;
    
    if keyvec.is_empty() {
        return vec![];
    }

    // Simple styling - just show the keys in angle brackets
    let key_string = keyvec
        .iter()
        .map(|k| k.to_string())
        .collect::<Vec<_>>()
        .join("|");

    vec![
        Style::new().paint("<"),
        Style::new().bold().paint(key_string),
        Style::new().paint(">"),
    ]
}
