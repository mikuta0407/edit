use std::path::PathBuf;

use edit::buffer::TextBuffer;
use edit::cell::{Ref, SemiRefCell};
use edit::input::{Action, KeyBindings, parse_key};
use edit::json;
use edit::lsh::{LANGUAGES, Language};
use stdext::arena::{read_to_string, scratch_arena};
use stdext::arena_format;

use crate::apperr;

pub struct Settings {
    pub path: PathBuf,
    pub file_associations: Vec<(String, &'static Language)>,
    /// Path to the user's `keybindings.json` (empty if there's no config dir).
    pub keybindings_path: PathBuf,
    /// Effective key bindings: built-in defaults plus the user's overrides.
    pub key_bindings: KeyBindings,
}

struct SettingsCell(SemiRefCell<Settings>);
unsafe impl Sync for SettingsCell {}
static SETTINGS: SettingsCell = SettingsCell(SemiRefCell::new(Settings::new()));

impl Settings {
    /// Fills the given settings.json text buffer with some initial contents for convenience.
    pub fn bootstrap(tb: &mut TextBuffer) {
        tb.set_crlf(false);
        tb.write_raw(b"{\n}\n");
        tb.cursor_move_to_logical(Default::default());
        tb.mark_as_clean();
    }

    /// Fills the given keybindings.json text buffer with some initial contents for convenience.
    pub fn bootstrap_keybindings(tb: &mut TextBuffer) {
        tb.set_crlf(false);
        tb.write_raw(b"{\n}\n");
        tb.cursor_move_to_logical(Default::default());
        tb.mark_as_clean();
    }

    const fn new() -> Self {
        Settings {
            path: PathBuf::new(),
            file_associations: Vec::new(),
            keybindings_path: PathBuf::new(),
            key_bindings: KeyBindings::new(),
        }
    }

    pub fn borrow() -> Ref<'static, Settings> {
        SETTINGS.0.borrow()
    }

    pub fn reload() -> apperr::Result<()> {
        let s = &mut *SETTINGS.0.borrow_mut();

        // Reset all members if we had been loaded previously.
        if !s.path.as_os_str().is_empty() {
            *s = Settings::new();
        }

        s.load()
    }

    fn load(&mut self) -> apperr::Result<()> {
        // Start from the built-in bindings, then layer user overrides on top.
        // This happens regardless of whether settings.json exists.
        self.key_bindings = KeyBindings::default();
        let keybindings_result = self.load_keybindings();

        // Load settings.json even if keybindings.json was invalid, and vice
        // versa; report the first error that occurred.
        let settings_result = self.load_settings_json();

        keybindings_result.and(settings_result)
    }

    fn load_keybindings(&mut self) -> apperr::Result<()> {
        self.keybindings_path = match keybindings_json_path() {
            Some(p) => p,
            None => return Ok(()),
        };

        let scratch = scratch_arena(None);
        let str = match read_to_string(&scratch, &self.keybindings_path) {
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(err) => return Err(err.into()),
            Ok(str) => str,
        };
        let Ok(json) = json::parse(&scratch, &str) else {
            return Err(apperr::Error::SettingsInvalid("Invalid JSON in keybindings.json"));
        };
        let Some(root) = json.as_object() else {
            return Err(apperr::Error::SettingsInvalid("keybindings.json: non-object root"));
        };

        for &(name, ref value) in root.iter() {
            let Some(action) = Action::from_name(name) else {
                return Err(apperr::Error::SettingsInvalid("keybindings.json: unknown action"));
            };

            let mut keys = Vec::new();
            if let Some(s) = value.as_str() {
                let Some(key) = parse_key(s) else {
                    return Err(apperr::Error::SettingsInvalid("keybindings.json: invalid key"));
                };
                keys.push(key);
            } else if let Some(arr) = value.as_array() {
                for v in arr {
                    let Some(s) = v.as_str() else {
                        return Err(apperr::Error::SettingsInvalid(
                            "keybindings.json: key must be a string",
                        ));
                    };
                    let Some(key) = parse_key(s) else {
                        return Err(apperr::Error::SettingsInvalid(
                            "keybindings.json: invalid key",
                        ));
                    };
                    keys.push(key);
                }
            } else {
                return Err(apperr::Error::SettingsInvalid(
                    "keybindings.json: value must be a string or array of strings",
                ));
            }

            self.key_bindings.apply_override(action, &keys);
        }

        Ok(())
    }

    fn load_settings_json(&mut self) -> apperr::Result<()> {
        self.path = match settings_json_path() {
            Some(p) => p,
            None => return Ok(()),
        };

        let scratch = scratch_arena(None);
        let str = match read_to_string(&scratch, &self.path) {
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(err) => return Err(err.into()),
            Ok(str) => str,
        };
        let Ok(json) = json::parse(&scratch, &str) else {
            return Err(apperr::Error::SettingsInvalid("Invalid JSON"));
        };
        let Some(root) = json.as_object() else {
            return Err(apperr::Error::SettingsInvalid("Non-object root"));
        };

        if let Some(f) = root.get_object("files.associations") {
            for &(mut key, ref value) in f.iter() {
                if !key.contains('/') {
                    key = arena_format!(&*scratch, "**/{key}").leak();
                }

                let Some(id) = value.as_str() else {
                    return Err(apperr::Error::SettingsInvalid("files.associations"));
                };
                let Some(language) = LANGUAGES.iter().find(|lang| lang.id == id) else {
                    return Err(apperr::Error::SettingsInvalid("language ID"));
                };

                self.file_associations.push((key.to_string(), language));
            }
        }

        Ok(())
    }
}

fn settings_json_path() -> Option<PathBuf> {
    let mut config_dir = config_dir()?;
    config_dir.push("settings.json");
    Some(config_dir)
}

fn keybindings_json_path() -> Option<PathBuf> {
    let mut config_dir = keybindings_config_dir()?;
    config_dir.push("keybindings.json");
    Some(config_dir)
}

/// Resolves the directory for `keybindings.json`.
///
/// Unlike [`config_dir`], this intentionally uses `~/.config/edit` on all
/// Unix-like systems (including macOS) so the location is predictable and
/// matches the documented configuration path.
fn keybindings_config_dir() -> Option<PathBuf> {
    fn var_path(key: &str) -> Option<PathBuf> {
        std::env::var_os(key).map(PathBuf::from)
    }

    fn push(mut path: PathBuf, suffix: &str) -> PathBuf {
        path.push(suffix);
        path
    }

    #[cfg(target_os = "windows")]
    {
        var_path("APPDATA").map(|p| push(p, "edit"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        var_path("XDG_CONFIG_HOME")
            .or_else(|| var_path("HOME").map(|p| push(p, ".config")))
            .map(|p| push(p, "edit"))
    }
}

fn config_dir() -> Option<PathBuf> {
    fn var_path(key: &str) -> Option<PathBuf> {
        std::env::var_os(key).map(PathBuf::from)
    }

    fn push(mut path: PathBuf, suffix: &str) -> PathBuf {
        path.push(suffix);
        path
    }

    #[cfg(target_os = "windows")]
    {
        var_path("APPDATA").map(|p| push(p, "Microsoft\\Edit"))
    }
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        var_path("HOME").map(|p| push(p, "Library/Application Support/com.microsoft.edit"))
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "ios")))]
    {
        var_path("XDG_CONFIG_HOME")
            .or_else(|| var_path("HOME").map(|p| push(p, ".config")))
            .map(|p| push(p, "msedit"))
    }
}
