// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Parses VT sequences into input events.
//!
//! In the future this allows us to take apart the application and
//! support input schemes that aren't VT, such as UEFI, or GUI.

use std::mem;

use crate::helpers::{CoordType, Point, Size};
use crate::vt;

/// Represents a key/modifier combination.
///
/// TODO: Is this a good idea? I did it to allow typing `kbmod::CTRL | vk::A`.
/// The reason it's an awkward u32 and not a struct is to hopefully make ABIs easier later.
/// Of course you could just translate on the ABI boundary, but my hope is that this
/// design lets me realize some restrictions early on that I can't foresee yet.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct InputKey(u32);

impl InputKey {
    pub(crate) const fn new(v: u32) -> Self {
        Self(v)
    }

    pub(crate) const fn from_ascii(ch: char) -> Option<Self> {
        if ch == ' ' || (ch >= '0' && ch <= '9') {
            Some(Self(ch as u32))
        } else if ch >= 'a' && ch <= 'z' {
            Some(Self(ch as u32 & !0x20)) // Shift a-z to A-Z
        } else if ch >= 'A' && ch <= 'Z' {
            Some(Self(kbmod::SHIFT.0 | ch as u32))
        } else {
            None
        }
    }

    pub(crate) const fn value(&self) -> u32 {
        self.0
    }

    pub(crate) const fn key(&self) -> Self {
        Self(self.0 & 0x00FFFFFF)
    }

    pub(crate) const fn modifiers(&self) -> InputKeyMod {
        InputKeyMod(self.0 & 0xFF000000)
    }

    pub(crate) const fn modifiers_contains(&self, modifier: InputKeyMod) -> bool {
        (self.0 & modifier.0) != 0
    }

    pub(crate) const fn with_modifiers(&self, modifiers: InputKeyMod) -> Self {
        Self(self.0 | modifiers.0)
    }
}

/// A keyboard modifier. Ctrl/Alt/Shift.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct InputKeyMod(u32);

impl InputKeyMod {
    const fn new(v: u32) -> Self {
        Self(v)
    }

    pub(crate) const fn contains(&self, modifier: Self) -> bool {
        (self.0 & modifier.0) != 0
    }
}

impl std::ops::BitOr<InputKeyMod> for InputKey {
    type Output = Self;

    fn bitor(self, rhs: InputKeyMod) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOr<InputKey> for InputKeyMod {
    type Output = InputKey;

    fn bitor(self, rhs: InputKey) -> InputKey {
        InputKey(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for InputKeyMod {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Keyboard keys.
///
/// The codes defined here match the VK_* constants on Windows.
/// It's a convenient way to handle keyboard input, even on other platforms.
pub mod vk {
    use super::InputKey;

    pub const NULL: InputKey = InputKey::new('\0' as u32);
    pub const BACK: InputKey = InputKey::new(0x08);
    pub const TAB: InputKey = InputKey::new('\t' as u32);
    pub const RETURN: InputKey = InputKey::new('\r' as u32);
    pub const ESCAPE: InputKey = InputKey::new(0x1B);
    pub const SPACE: InputKey = InputKey::new(' ' as u32);
    pub const PRIOR: InputKey = InputKey::new(0x21);
    pub const NEXT: InputKey = InputKey::new(0x22);

    pub const END: InputKey = InputKey::new(0x23);
    pub const HOME: InputKey = InputKey::new(0x24);

    pub const LEFT: InputKey = InputKey::new(0x25);
    pub const UP: InputKey = InputKey::new(0x26);
    pub const RIGHT: InputKey = InputKey::new(0x27);
    pub const DOWN: InputKey = InputKey::new(0x28);

    pub const INSERT: InputKey = InputKey::new(0x2D);
    pub const DELETE: InputKey = InputKey::new(0x2E);

    pub const N0: InputKey = InputKey::new('0' as u32);
    pub const N1: InputKey = InputKey::new('1' as u32);
    pub const N2: InputKey = InputKey::new('2' as u32);
    pub const N3: InputKey = InputKey::new('3' as u32);
    pub const N4: InputKey = InputKey::new('4' as u32);
    pub const N5: InputKey = InputKey::new('5' as u32);
    pub const N6: InputKey = InputKey::new('6' as u32);
    pub const N7: InputKey = InputKey::new('7' as u32);
    pub const N8: InputKey = InputKey::new('8' as u32);
    pub const N9: InputKey = InputKey::new('9' as u32);

    pub const A: InputKey = InputKey::new('A' as u32);
    pub const B: InputKey = InputKey::new('B' as u32);
    pub const C: InputKey = InputKey::new('C' as u32);
    pub const D: InputKey = InputKey::new('D' as u32);
    pub const E: InputKey = InputKey::new('E' as u32);
    pub const F: InputKey = InputKey::new('F' as u32);
    pub const G: InputKey = InputKey::new('G' as u32);
    pub const H: InputKey = InputKey::new('H' as u32);
    pub const I: InputKey = InputKey::new('I' as u32);
    pub const J: InputKey = InputKey::new('J' as u32);
    pub const K: InputKey = InputKey::new('K' as u32);
    pub const L: InputKey = InputKey::new('L' as u32);
    pub const M: InputKey = InputKey::new('M' as u32);
    pub const N: InputKey = InputKey::new('N' as u32);
    pub const O: InputKey = InputKey::new('O' as u32);
    pub const P: InputKey = InputKey::new('P' as u32);
    pub const Q: InputKey = InputKey::new('Q' as u32);
    pub const R: InputKey = InputKey::new('R' as u32);
    pub const S: InputKey = InputKey::new('S' as u32);
    pub const T: InputKey = InputKey::new('T' as u32);
    pub const U: InputKey = InputKey::new('U' as u32);
    pub const V: InputKey = InputKey::new('V' as u32);
    pub const W: InputKey = InputKey::new('W' as u32);
    pub const X: InputKey = InputKey::new('X' as u32);
    pub const Y: InputKey = InputKey::new('Y' as u32);
    pub const Z: InputKey = InputKey::new('Z' as u32);

    pub const NUMPAD0: InputKey = InputKey::new(0x60);
    pub const NUMPAD1: InputKey = InputKey::new(0x61);
    pub const NUMPAD2: InputKey = InputKey::new(0x62);
    pub const NUMPAD3: InputKey = InputKey::new(0x63);
    pub const NUMPAD4: InputKey = InputKey::new(0x64);
    pub const NUMPAD5: InputKey = InputKey::new(0x65);
    pub const NUMPAD6: InputKey = InputKey::new(0x66);
    pub const NUMPAD7: InputKey = InputKey::new(0x67);
    pub const NUMPAD8: InputKey = InputKey::new(0x68);
    pub const NUMPAD9: InputKey = InputKey::new(0x69);
    pub const MULTIPLY: InputKey = InputKey::new(0x6A);
    pub const ADD: InputKey = InputKey::new(0x6B);
    pub const SEPARATOR: InputKey = InputKey::new(0x6C);
    pub const SUBTRACT: InputKey = InputKey::new(0x6D);
    pub const DECIMAL: InputKey = InputKey::new(0x6E);
    pub const DIVIDE: InputKey = InputKey::new(0x6F);

    pub const F1: InputKey = InputKey::new(0x70);
    pub const F2: InputKey = InputKey::new(0x71);
    pub const F3: InputKey = InputKey::new(0x72);
    pub const F4: InputKey = InputKey::new(0x73);
    pub const F5: InputKey = InputKey::new(0x74);
    pub const F6: InputKey = InputKey::new(0x75);
    pub const F7: InputKey = InputKey::new(0x76);
    pub const F8: InputKey = InputKey::new(0x77);
    pub const F9: InputKey = InputKey::new(0x78);
    pub const F10: InputKey = InputKey::new(0x79);
    pub const F11: InputKey = InputKey::new(0x7A);
    pub const F12: InputKey = InputKey::new(0x7B);
    pub const F13: InputKey = InputKey::new(0x7C);
    pub const F14: InputKey = InputKey::new(0x7D);
    pub const F15: InputKey = InputKey::new(0x7E);
    pub const F16: InputKey = InputKey::new(0x7F);
    pub const F17: InputKey = InputKey::new(0x80);
    pub const F18: InputKey = InputKey::new(0x81);
    pub const F19: InputKey = InputKey::new(0x82);
    pub const F20: InputKey = InputKey::new(0x83);
    pub const F21: InputKey = InputKey::new(0x84);
    pub const F22: InputKey = InputKey::new(0x85);
    pub const F23: InputKey = InputKey::new(0x86);
    pub const F24: InputKey = InputKey::new(0x87);
}

/// Keyboard modifiers.
pub mod kbmod {
    use super::InputKeyMod;

    pub const NONE: InputKeyMod = InputKeyMod::new(0x00000000);
    pub const CTRL: InputKeyMod = InputKeyMod::new(0x01000000);
    pub const ALT: InputKeyMod = InputKeyMod::new(0x02000000);
    pub const SHIFT: InputKeyMod = InputKeyMod::new(0x04000000);

    pub const CTRL_ALT: InputKeyMod = InputKeyMod::new(0x03000000);
    pub const CTRL_SHIFT: InputKeyMod = InputKeyMod::new(0x05000000);
    pub const ALT_SHIFT: InputKeyMod = InputKeyMod::new(0x06000000);
    pub const CTRL_ALT_SHIFT: InputKeyMod = InputKeyMod::new(0x07000000);
}

/// An editor command that a key can be bound to.
///
/// Variants are split into "application" actions (handled by the binary's
/// global shortcut dispatch) and "textarea" actions (handled by the textarea
/// in [`crate::tui`]). The split is purely about *who executes them*; both
/// share a single [`KeyBindings`] map so users configure them in one place.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    // Application actions (handled by the binary).
    FileNew,
    FileOpen,
    FileSave,
    FileSaveAs,
    FileClose,
    FileExit,
    GoToFile,
    GoToLine,
    Find,
    Replace,
    FindNext,

    // Textarea actions (handled by `tui`).
    SelectAll,
    SelectLine,
    Copy,
    Cut,
    Paste,
    Undo,
    Redo,
    DeleteWordLeft,
    DeleteWordRight,
    ToggleWordWrap,
    ToggleOvertype,
    WordLeft,
    WordRight,
    LineStart,
    LineEnd,
    DocumentStart,
    DocumentEnd,
}

impl Action {
    /// Maps a configuration name (e.g. `"selectAll"`) to an [`Action`].
    pub fn from_name(name: &str) -> Option<Action> {
        Some(match name {
            "new" => Action::FileNew,
            "open" => Action::FileOpen,
            "save" => Action::FileSave,
            "saveAs" => Action::FileSaveAs,
            "close" => Action::FileClose,
            "exit" => Action::FileExit,
            "goToFile" => Action::GoToFile,
            "goToLine" => Action::GoToLine,
            "find" => Action::Find,
            "replace" => Action::Replace,
            "findNext" => Action::FindNext,
            "selectAll" => Action::SelectAll,
            "selectLine" => Action::SelectLine,
            "copy" => Action::Copy,
            "cut" => Action::Cut,
            "paste" => Action::Paste,
            "undo" => Action::Undo,
            "redo" => Action::Redo,
            "deleteWordLeft" => Action::DeleteWordLeft,
            "deleteWordRight" => Action::DeleteWordRight,
            "toggleWordWrap" => Action::ToggleWordWrap,
            "toggleOvertype" => Action::ToggleOvertype,
            "wordLeft" => Action::WordLeft,
            "wordRight" => Action::WordRight,
            "lineStart" => Action::LineStart,
            "lineEnd" => Action::LineEnd,
            "documentStart" => Action::DocumentStart,
            "documentEnd" => Action::DocumentEnd,
            _ => return None,
        })
    }
}

/// Parses a key combination string such as `"ctrl+shift+a"`, `"alt+z"`,
/// `"home"` or `"f3"` into an [`InputKey`]. Returns `None` if the string is
/// not a valid combination. Parsing is case-insensitive and modifier order
/// independent.
pub fn parse_key(s: &str) -> Option<InputKey> {
    let mut modifiers = kbmod::NONE;
    let mut key: Option<InputKey> = None;

    for part in s.split('+') {
        let part = part.trim();
        if part.is_empty() {
            return None;
        }

        match part.to_ascii_lowercase().as_str() {
            "ctrl" | "control" | "ctl" => modifiers |= kbmod::CTRL,
            "alt" | "option" | "opt" | "meta" => modifiers |= kbmod::ALT,
            "shift" => modifiers |= kbmod::SHIFT,
            name => {
                // Only a single non-modifier key is allowed.
                if key.is_some() {
                    return None;
                }
                key = Some(parse_key_name(name)?);
            }
        }
    }

    Some(key?.with_modifiers(modifiers))
}

/// Parses the non-modifier portion of a key combination (already lowercased).
fn parse_key_name(s: &str) -> Option<InputKey> {
    let bytes = s.as_bytes();
    if bytes.len() == 1 {
        let c = bytes[0];
        if c.is_ascii_lowercase() {
            // `vk::A` etc. use the uppercase ASCII value.
            return Some(InputKey::new((c - 0x20) as u32));
        }
        if c.is_ascii_digit() {
            return Some(InputKey::new(c as u32));
        }
    }

    Some(match s {
        "home" => vk::HOME,
        "end" => vk::END,
        "left" => vk::LEFT,
        "right" => vk::RIGHT,
        "up" => vk::UP,
        "down" => vk::DOWN,
        "pageup" | "prior" => vk::PRIOR,
        "pagedown" | "next" => vk::NEXT,
        "insert" | "ins" => vk::INSERT,
        "delete" | "del" => vk::DELETE,
        "backspace" | "back" | "bksp" => vk::BACK,
        "tab" => vk::TAB,
        "enter" | "return" => vk::RETURN,
        "escape" | "esc" => vk::ESCAPE,
        "space" => vk::SPACE,
        "f1" => vk::F1,
        "f2" => vk::F2,
        "f3" => vk::F3,
        "f4" => vk::F4,
        "f5" => vk::F5,
        "f6" => vk::F6,
        "f7" => vk::F7,
        "f8" => vk::F8,
        "f9" => vk::F9,
        "f10" => vk::F10,
        "f11" => vk::F11,
        "f12" => vk::F12,
        "f13" => vk::F13,
        "f14" => vk::F14,
        "f15" => vk::F15,
        "f16" => vk::F16,
        "f17" => vk::F17,
        "f18" => vk::F18,
        "f19" => vk::F19,
        "f20" => vk::F20,
        "f21" => vk::F21,
        "f22" => vk::F22,
        "f23" => vk::F23,
        "f24" => vk::F24,
        _ => return None,
    })
}

/// A mapping of key combinations to [`Action`]s.
///
/// Built from a set of built-in defaults (see [`KeyBindings::default`]) that
/// reproduce the editor's classic behavior, then optionally customized via
/// [`KeyBindings::apply_override`] from the user's `keybindings.json`.
#[derive(Clone)]
pub struct KeyBindings {
    map: Vec<(InputKey, Action)>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        let mut map = vec![
            // Application actions.
            (vk::N.with_modifiers(kbmod::CTRL), Action::FileNew),
            (vk::O.with_modifiers(kbmod::CTRL), Action::FileOpen),
            (vk::S.with_modifiers(kbmod::CTRL), Action::FileSave),
            (vk::S.with_modifiers(kbmod::CTRL_SHIFT), Action::FileSaveAs),
            (vk::W.with_modifiers(kbmod::CTRL), Action::FileClose),
            (vk::Q.with_modifiers(kbmod::CTRL), Action::FileExit),
            (vk::P.with_modifiers(kbmod::CTRL), Action::GoToFile),
            (vk::G.with_modifiers(kbmod::CTRL), Action::GoToLine),
            (vk::F.with_modifiers(kbmod::CTRL), Action::Find),
            (vk::R.with_modifiers(kbmod::CTRL), Action::Replace),
            (vk::F3, Action::FindNext),
            // Textarea actions.
            (vk::A.with_modifiers(kbmod::CTRL), Action::SelectAll),
            (vk::L.with_modifiers(kbmod::CTRL), Action::SelectLine),
            (vk::C.with_modifiers(kbmod::CTRL), Action::Copy),
            (vk::INSERT.with_modifiers(kbmod::CTRL), Action::Copy),
            (vk::X.with_modifiers(kbmod::CTRL), Action::Cut),
            (vk::DELETE.with_modifiers(kbmod::SHIFT), Action::Cut),
            (vk::V.with_modifiers(kbmod::CTRL), Action::Paste),
            (vk::INSERT.with_modifiers(kbmod::SHIFT), Action::Paste),
            (vk::Z.with_modifiers(kbmod::CTRL), Action::Undo),
            (vk::Y.with_modifiers(kbmod::CTRL), Action::Redo),
            (vk::Z.with_modifiers(kbmod::CTRL_SHIFT), Action::Redo),
            (vk::H.with_modifiers(kbmod::CTRL), Action::DeleteWordLeft),
            (vk::DELETE.with_modifiers(kbmod::CTRL), Action::DeleteWordRight),
            (vk::Z.with_modifiers(kbmod::ALT), Action::ToggleWordWrap),
            (vk::INSERT, Action::ToggleOvertype),
        ];

        // On macOS, terminals commonly emit the Emacs style Alt+B/Alt+F
        // (ESC b / ESC f) sequences for word-wise cursor movement.
        if cfg!(any(target_os = "macos", target_os = "ios")) {
            map.push((vk::B.with_modifiers(kbmod::ALT), Action::WordLeft));
            map.push((vk::F.with_modifiers(kbmod::ALT), Action::WordRight));
        }

        Self { map }
    }
}

impl KeyBindings {
    /// Creates an empty set of key bindings (no defaults).
    ///
    /// Useful as a `const` initializer; call [`KeyBindings::default`] for the
    /// built-in bindings.
    pub const fn new() -> Self {
        Self { map: Vec::new() }
    }

    /// Rebinds `action` to exactly `keys`, replacing any previous keys bound to
    /// it and taking over those keys from any other action (last write wins).
    pub fn apply_override(&mut self, action: Action, keys: &[InputKey]) {
        // Free the keys this action was previously bound to.
        self.map.retain(|&(_, a)| a != action);
        for &key in keys {
            // The newly assigned key takes precedence over any other action.
            self.map.retain(|&(k, _)| k != key);
            self.map.push((key, action));
        }
    }

    /// Returns the action bound to `key`, if any.
    pub fn action_for(&self, key: InputKey) -> Option<Action> {
        self.map.iter().find(|&&(k, _)| k == key).map(|&(_, a)| a)
    }

    /// Returns a representative key bound to `action` (for display purposes).
    pub fn key_for(&self, action: Action) -> Option<InputKey> {
        self.map.iter().find(|&&(_, a)| a == action).map(|&(k, _)| k)
    }
}

/// Mouse input state. Up/Down, Left/Right, etc.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum InputMouseState {
    #[default]
    None,

    // These 3 carry their state between frames.
    Left,
    Middle,
    Right,

    // These 2 get reset to None on the next frame.
    Release,
    Scroll,
}

/// Mouse input.
#[derive(Clone, Copy)]
pub struct InputMouse {
    /// The state of the mouse.Up/Down, Left/Right, etc.
    pub state: InputMouseState,
    /// Any keyboard modifiers that are held down.
    pub modifiers: InputKeyMod,
    /// Position of the mouse in the viewport.
    pub position: Point,
    /// Scroll delta.
    pub scroll: Point,
    /// Whether the mouse is being dragged with a button held down.
    pub drag: bool,
}

/// Primary result type of the parser.
pub enum Input<'input> {
    /// Window resize event.
    Resize(Size),
    /// Text input.
    /// Note that [`Input::Keyboard`] events can also be text.
    Text(&'input str),
    /// A clipboard paste.
    Paste(Vec<u8>),
    /// Keyboard input.
    Keyboard(InputKey),
    /// Mouse input.
    Mouse(InputMouse),
}

/// Parses VT sequences into input events.
pub struct Parser {
    bracketed_paste: bool,
    bracketed_paste_buf: Vec<u8>,
    x10_mouse_want: bool,
    x10_mouse_buf: [char; 3],
    x10_mouse_len: usize,
}

impl Parser {
    /// Creates a new parser that turns VT sequences into input events.
    ///
    /// Keep the instance alive for the lifetime of the input stream.
    pub fn new() -> Self {
        Self {
            bracketed_paste: false,
            bracketed_paste_buf: Vec::new(),
            x10_mouse_want: false,
            x10_mouse_buf: ['\0'; 3],
            x10_mouse_len: 0,
        }
    }

    /// Takes an [`vt::Stream`] and returns a [`Stream`]
    /// that turns VT sequences into input events.
    pub fn parse<'parser, 'vt, 'input>(
        &'parser mut self,
        stream: vt::Stream<'vt, 'input>,
    ) -> Stream<'parser, 'vt, 'input> {
        Stream { parser: self, stream }
    }
}

/// An iterator that parses VT sequences into input events.
pub struct Stream<'parser, 'vt, 'input> {
    parser: &'parser mut Parser,
    stream: vt::Stream<'vt, 'input>,
}

impl<'input> Iterator for Stream<'_, '_, 'input> {
    type Item = Input<'input>;

    fn next(&mut self) -> Option<Input<'input>> {
        loop {
            if self.parser.bracketed_paste {
                return self.handle_bracketed_paste();
            }

            if self.parser.x10_mouse_want {
                return self.parse_x10_mouse_coordinates();
            }

            const KEYPAD_LUT: [u8; 8] = [
                vk::UP.value() as u8,    // A
                vk::DOWN.value() as u8,  // B
                vk::RIGHT.value() as u8, // C
                vk::LEFT.value() as u8,  // D
                0,                       // E
                vk::END.value() as u8,   // F
                0,                       // G
                vk::HOME.value() as u8,  // H
            ];

            match self.stream.next()? {
                vt::Token::Text(text) => {
                    return Some(Input::Text(text));
                }
                vt::Token::Ctrl(ch) => match ch {
                    '\0' | '\t' | '\r' => return Some(Input::Keyboard(InputKey::new(ch as u32))),
                    '\n' => return Some(Input::Keyboard(kbmod::CTRL | vk::RETURN)),
                    ..='\x1a' => {
                        // Shift control code to A-Z
                        let key = ch as u32 | 0x40;
                        return Some(Input::Keyboard(kbmod::CTRL | InputKey::new(key)));
                    }
                    '\x7f' => return Some(Input::Keyboard(vk::BACK)),
                    _ => {}
                },
                vt::Token::Esc(ch) => {
                    match ch {
                        '\0' => return Some(Input::Keyboard(vk::ESCAPE)),
                        '\n' => return Some(Input::Keyboard(kbmod::CTRL_ALT | vk::RETURN)),
                        ' '..='~' => {
                            let ch = ch as u32;
                            let key = ch & !0x20; // Shift a-z to A-Z
                            let modifiers =
                                if (ch & 0x20) != 0 { kbmod::ALT } else { kbmod::ALT_SHIFT };
                            return Some(Input::Keyboard(modifiers | InputKey::new(key)));
                        }
                        _ => {}
                    }
                }
                vt::Token::SS3(ch) => match ch {
                    'A'..='H' => {
                        let vk = KEYPAD_LUT[ch as usize - 'A' as usize];
                        if vk != 0 {
                            return Some(Input::Keyboard(InputKey::new(vk as u32)));
                        }
                    }
                    'P'..='S' => {
                        let key = vk::F1.value() + ch as u32 - 'P' as u32;
                        return Some(Input::Keyboard(InputKey::new(key)));
                    }
                    _ => {}
                },
                vt::Token::Csi(csi) => {
                    match csi.final_byte {
                        'A'..='H' => {
                            let vk = KEYPAD_LUT[csi.final_byte as usize - 'A' as usize];
                            if vk != 0 {
                                return Some(Input::Keyboard(
                                    InputKey::new(vk as u32) | Self::parse_modifiers(csi),
                                ));
                            }
                        }
                        'Z' => return Some(Input::Keyboard(kbmod::SHIFT | vk::TAB)),
                        '~' => {
                            const LUT: [u8; 35] = [
                                0,
                                vk::HOME.value() as u8,   // 1
                                vk::INSERT.value() as u8, // 2
                                vk::DELETE.value() as u8, // 3
                                vk::END.value() as u8,    // 4
                                vk::PRIOR.value() as u8,  // 5
                                vk::NEXT.value() as u8,   // 6
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                                vk::F5.value() as u8, // 15
                                0,
                                vk::F6.value() as u8,  // 17
                                vk::F7.value() as u8,  // 18
                                vk::F8.value() as u8,  // 19
                                vk::F9.value() as u8,  // 20
                                vk::F10.value() as u8, // 21
                                0,
                                vk::F11.value() as u8, // 23
                                vk::F12.value() as u8, // 24
                                vk::F13.value() as u8, // 25
                                vk::F14.value() as u8, // 26
                                0,
                                vk::F15.value() as u8, // 28
                                vk::F16.value() as u8, // 29
                                0,
                                vk::F17.value() as u8, // 31
                                vk::F18.value() as u8, // 32
                                vk::F19.value() as u8, // 33
                                vk::F20.value() as u8, // 34
                            ];
                            const LUT_LEN: u16 = LUT.len() as u16;

                            match csi.params[0] {
                                0..LUT_LEN => {
                                    let vk = LUT[csi.params[0] as usize];
                                    if vk != 0 {
                                        return Some(Input::Keyboard(
                                            InputKey::new(vk as u32) | Self::parse_modifiers(csi),
                                        ));
                                    }
                                }
                                200 => self.parser.bracketed_paste = true,
                                _ => {}
                            }
                        }
                        'm' | 'M' if csi.private_byte == '<' => {
                            return Self::parse_xterm_mouse(
                                &csi.params[..csi.param_count],
                                csi.final_byte,
                            );
                        }
                        'M' if csi.param_count == 0 => {
                            self.parser.x10_mouse_want = true;
                        }
                        't' if csi.params[0] == 8 => {
                            // Window Size
                            let width = (csi.params[2] as CoordType).clamp(1, 32767);
                            let height = (csi.params[1] as CoordType).clamp(1, 32767);
                            return Some(Input::Resize(Size { width, height }));
                        }
                        'u' => {
                            // Kitty keyboard protocol key event:
                            // `CSI <codepoint> ; <modifiers> u`.
                            if let Some(input) = Self::parse_kitty_key(csi) {
                                return Some(input);
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}

impl<'input> Stream<'_, '_, 'input> {
    /// Once we encounter the start of a bracketed paste
    /// we seek to the end of the paste in this function.
    ///
    /// A bracketed paste is basically:
    /// ```text
    /// <ESC>[201~    lots of text    <ESC>[201~
    /// ```
    ///
    /// That in between text is then expected to be taken literally.
    /// It can be in between anything though, including other escape sequences.
    /// This is the reason why this is a separate method.
    #[cold]
    fn handle_bracketed_paste(&mut self) -> Option<Input<'input>> {
        let beg = self.stream.offset();
        let mut end = beg;

        while let Some(token) = self.stream.next() {
            if let vt::Token::Csi(csi) = token
                && csi.final_byte == '~'
                && csi.params[0] == 201
            {
                self.parser.bracketed_paste = false;
                break;
            }
            end = self.stream.offset();
        }

        if end != beg {
            self.parser
                .bracketed_paste_buf
                .extend_from_slice(&self.stream.input().as_bytes()[beg..end]);
        }

        if !self.parser.bracketed_paste {
            Some(Input::Paste(mem::take(&mut self.parser.bracketed_paste_buf)))
        } else {
            None
        }
    }

    /// Implements the X10 mouse protocol via `CSI M CbCxCy`.
    ///
    /// You want to send numeric mouse coordinates.
    /// You have CSI sequences with numeric parameters.
    /// So, of course you put the coordinates as shifted ASCII characters after
    /// the end of the sequence. Limited coordinate range and complicated parsing!
    /// This is so puzzling to me. The existence of this function makes me unhappy.
    #[cold]
    fn parse_x10_mouse_coordinates(&mut self) -> Option<Input<'input>> {
        while self.parser.x10_mouse_len < 3 && !self.stream.done() {
            self.parser.x10_mouse_buf[self.parser.x10_mouse_len] = self.stream.next_char();
            self.parser.x10_mouse_len += 1;
        }
        if self.parser.x10_mouse_len < 3 {
            return None;
        }

        let b = self.parser.x10_mouse_buf[0] as u16 - 0x20;
        let x = self.parser.x10_mouse_buf[1] as u16 - 0x20;
        let y = self.parser.x10_mouse_buf[2] as u16 - 0x20;

        self.parser.x10_mouse_want = false;
        self.parser.x10_mouse_len = 0;

        Self::parse_xterm_mouse(&[b, x, y], 'M')
    }

    fn parse_modifiers(csi: &vt::Csi) -> InputKeyMod {
        let mut modifiers = kbmod::NONE;
        let p1 = csi.params[1].saturating_sub(1);
        if (p1 & 0x01) != 0 {
            modifiers |= kbmod::SHIFT;
        }
        if (p1 & 0x02) != 0 {
            modifiers |= kbmod::ALT;
        }
        if (p1 & 0x04) != 0 {
            modifiers |= kbmod::CTRL;
        }
        modifiers
    }

    /// Parses a Kitty keyboard protocol key event (`CSI <codepoint> ; <mods> u`).
    ///
    /// The codepoint is the base (unshifted) key; modifiers (including Shift)
    /// are carried separately, which is what lets us distinguish e.g.
    /// `Ctrl+Shift+A` from `Ctrl+A` — something the legacy control-code
    /// encoding cannot express. We only enable the "disambiguate" flag, so
    /// functional keys (arrows, Home/End, F-keys, ...) keep arriving via their
    /// legacy CSI sequences and only "text-like" keys reach this function.
    fn parse_kitty_key(csi: &vt::Csi) -> Option<Input<'input>> {
        let codepoint = csi.params[0] as u32;
        let key = match codepoint {
            // Letters: map to the uppercase value `vk` uses (e.g. 'a' -> 'A').
            0x61..=0x7a => codepoint - 0x20,
            // Keys Kitty reports via their ASCII control codepoint.
            9 => vk::TAB.value(),
            13 => vk::RETURN.value(),
            27 => vk::ESCAPE.value(),
            127 => vk::BACK.value(),
            // Other plain ASCII keys (digits, space, symbols) pass through.
            0x20..=0x7e => codepoint,
            // Functional keys with Private Use Area codepoints, etc.: ignore.
            _ => return None,
        };
        Some(Input::Keyboard(InputKey::new(key) | Self::parse_modifiers(csi)))
    }

    fn parse_xterm_mouse(params: &[u16], final_byte: char) -> Option<Input<'input>> {
        const SHIFT: u16 = 0x04;
        const ALT: u16 = 0x08;
        const CTRL: u16 = 0x10;
        const MOTION: u16 = 0x20;
        const WHEEL: u16 = 0x40;
        const MODIFIERS: u16 = SHIFT | ALT | CTRL;

        let &[btn, x, y, ..] = params else {
            return None;
        };

        let kind = btn & !MODIFIERS;
        let x = x as CoordType - 1;
        let y = y as CoordType - 1;
        let mut mouse = InputMouse {
            state: InputMouseState::None,
            modifiers: kbmod::NONE,
            position: Point { x, y },
            scroll: Default::default(),
            drag: false,
        };

        if final_byte == 'm' {
            // M = down, m = release.
            // I know there's an InputMouseState::Release, but that's because the internals of tui.rs
            // have leaked into intput.rs. input.rs indicates release by the absence of buttons being
            // held, which is InputMouseState::None. This makes it more reliable in my opinion.
        } else if (WHEEL..WHEEL + 4).contains(&kind) {
            let delta = if (kind & 1) != 0 { 3 } else { -3 };
            let idx = if (kind & 2) != 0 { 0 } else { 1 };
            mouse.scroll.as_array()[idx] += delta;
            mouse.state = InputMouseState::Scroll;
        } else if (kind & !MOTION) < 3 {
            match kind & 3 {
                0 => mouse.state = InputMouseState::Left,
                1 => mouse.state = InputMouseState::Middle,
                2 => mouse.state = InputMouseState::Right,
                _ => {}
            }
            mouse.drag = (kind & MOTION) != 0;
        }

        mouse.modifiers = kbmod::NONE;
        mouse.modifiers |= if (btn & SHIFT) != 0 { kbmod::SHIFT } else { kbmod::NONE };
        mouse.modifiers |= if (btn & ALT) != 0 { kbmod::ALT } else { kbmod::NONE };
        mouse.modifiers |= if (btn & CTRL) != 0 { kbmod::CTRL } else { kbmod::NONE };

        Some(Input::Mouse(mouse))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs a raw byte sequence through the VT + input parsers and returns the
    /// first keyboard event, if any.
    fn first_key(seq: &str) -> Option<InputKey> {
        let mut vt_parser = crate::vt::Parser::new();
        let mut input_parser = Parser::new();
        let mut stream = input_parser.parse(vt_parser.parse(seq));
        match stream.next() {
            Some(Input::Keyboard(key)) => Some(key),
            _ => None,
        }
    }

    #[test]
    fn kitty_csi_u_distinguishes_ctrl_shift() {
        // `CSI <codepoint> ; <modifiers> u`, where modifiers = 1 + bitmask
        // (shift=1, alt=2, ctrl=4). Codepoint 97 = 'a', 101 = 'e'.
        assert_eq!(first_key("\x1b[97;5u"), Some(vk::A.with_modifiers(kbmod::CTRL)));
        assert_eq!(first_key("\x1b[97;6u"), Some(vk::A.with_modifiers(kbmod::CTRL_SHIFT)));
        assert_eq!(first_key("\x1b[101;6u"), Some(vk::E.with_modifiers(kbmod::CTRL_SHIFT)));
        // No-modifier and named keys.
        assert_eq!(first_key("\x1b[97u"), Some(vk::A));
        assert_eq!(first_key("\x1b[27u"), Some(vk::ESCAPE));
        assert_eq!(first_key("\x1b[13;5u"), Some(vk::RETURN.with_modifiers(kbmod::CTRL)));
    }

    #[test]
    fn parse_key_combinations() {
        assert_eq!(parse_key("ctrl+a"), Some(vk::A.with_modifiers(kbmod::CTRL)));
        assert_eq!(parse_key("Ctrl+Shift+A"), Some(vk::A.with_modifiers(kbmod::CTRL_SHIFT)));
        assert_eq!(parse_key("alt+z"), Some(vk::Z.with_modifiers(kbmod::ALT)));
        assert_eq!(parse_key("home"), Some(vk::HOME));
        assert_eq!(parse_key("f3"), Some(vk::F3));
        assert_eq!(parse_key("CTRL+SHIFT+F12"), Some(vk::F12.with_modifiers(kbmod::CTRL_SHIFT)));
        // Modifier order is irrelevant.
        assert_eq!(parse_key("shift+ctrl+a"), parse_key("ctrl+shift+a"));

        // Invalid inputs.
        assert_eq!(parse_key(""), None);
        assert_eq!(parse_key("ctrl+"), None);
        assert_eq!(parse_key("ctrl+nope"), None);
        assert_eq!(parse_key("ctrl+a+b"), None);
    }

    #[test]
    fn action_names() {
        assert_eq!(Action::from_name("selectAll"), Some(Action::SelectAll));
        assert_eq!(Action::from_name("lineStart"), Some(Action::LineStart));
        assert_eq!(Action::from_name("save"), Some(Action::FileSave));
        assert_eq!(Action::from_name("unknown"), None);
    }

    #[test]
    fn default_bindings_match_classic_behavior() {
        let kb = KeyBindings::default();
        assert_eq!(kb.action_for(vk::A.with_modifiers(kbmod::CTRL)), Some(Action::SelectAll));
        assert_eq!(kb.action_for(vk::S.with_modifiers(kbmod::CTRL)), Some(Action::FileSave));
        assert_eq!(kb.action_for(vk::C.with_modifiers(kbmod::CTRL)), Some(Action::Copy));
        // A single action can be reached via more than one key.
        assert_eq!(kb.action_for(vk::INSERT.with_modifiers(kbmod::CTRL)), Some(Action::Copy));
    }

    #[test]
    fn overrides_rebind_and_free_keys() {
        // Mirrors the documented example: move Select All to Ctrl+Shift+A and
        // bind Ctrl+A to "go to line start".
        let mut kb = KeyBindings::default();
        kb.apply_override(Action::SelectAll, &[parse_key("ctrl+shift+a").unwrap()]);
        kb.apply_override(Action::LineStart, &[parse_key("ctrl+a").unwrap()]);

        assert_eq!(kb.action_for(vk::A.with_modifiers(kbmod::CTRL)), Some(Action::LineStart));
        assert_eq!(kb.action_for(vk::A.with_modifiers(kbmod::CTRL_SHIFT)), Some(Action::SelectAll));
    }

    #[test]
    fn override_last_write_wins_on_key_collision() {
        // Binding a second action to Ctrl+A takes the key away from Select All
        // even without explicitly moving Select All first.
        let mut kb = KeyBindings::default();
        kb.apply_override(Action::LineStart, &[parse_key("ctrl+a").unwrap()]);
        assert_eq!(kb.action_for(vk::A.with_modifiers(kbmod::CTRL)), Some(Action::LineStart));
        assert_eq!(kb.key_for(Action::SelectAll), None);
    }
}
