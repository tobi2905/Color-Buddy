//! The picker hotkey, changeable at runtime (Settings and onboarding).

use std::str::FromStr;
use std::sync::Mutex;

use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

/// Ctrl+Shift+C on Windows, Cmd+Shift+C on macOS.
pub fn default_shortcut() -> Shortcut {
    #[cfg(target_os = "macos")]
    let mods = Modifiers::SUPER | Modifiers::SHIFT;
    #[cfg(not(target_os = "macos"))]
    let mods = Modifiers::CONTROL | Modifiers::SHIFT;
    Shortcut::new(Some(mods), Code::KeyC)
}

/// The currently registered picker shortcut.
pub struct Hotkey(Mutex<Shortcut>);

impl Default for Hotkey {
    fn default() -> Self {
        Self(Mutex::new(default_shortcut()))
    }
}

impl Hotkey {
    pub fn matches(&self, shortcut: &Shortcut) -> bool {
        *self.0.lock().unwrap() == *shortcut
    }
}

pub fn register_default(app: &AppHandle) {
    if let Err(err) = app.global_shortcut().register(default_shortcut()) {
        log::error!("could not register picker hotkey: {err}");
    }
}

/// Parse an accelerator such as "CommandOrControl+Shift+C".
pub fn parse(accelerator: &str) -> Result<Shortcut, String> {
    let shortcut = Shortcut::from_str(accelerator)
        .map_err(|e| format!("“{accelerator}” isn't a valid shortcut ({e})"))?;
    let strong = Modifiers::CONTROL | Modifiers::SUPER | Modifiers::ALT;
    if !shortcut.mods.intersects(strong) {
        return Err(
            "Use Ctrl, Cmd or Alt/Option in the shortcut so it doesn't block normal typing.".into(),
        );
    }
    Ok(shortcut)
}

/// Replace the picker shortcut. On failure the previous one stays registered.
pub fn set(app: &AppHandle, accelerator: &str) -> Result<(), String> {
    let new = parse(accelerator)?;
    let state = app.state::<Hotkey>();
    let mut current = state.0.lock().unwrap();
    if *current == new {
        return Ok(());
    }
    let gs = app.global_shortcut();
    let _ = gs.unregister(*current);
    if let Err(err) = gs.register(new) {
        let _ = gs.register(*current);
        return Err(format!(
            "That shortcut couldn't be registered, probably because another app uses it ({err})."
        ));
    }
    *current = new;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_accelerators() {
        let s = parse("CommandOrControl+Shift+C").unwrap();
        assert_eq!(s.key, Code::KeyC);
        assert!(s.mods.contains(Modifiers::SHIFT));
        assert!(parse("Alt+P").is_ok());
    }

    #[test]
    fn rejects_shortcuts_that_would_block_typing() {
        assert!(parse("Shift+C").is_err());
        assert!(parse("C").is_err());
        assert!(parse("Banana+Q").is_err());
    }
}
