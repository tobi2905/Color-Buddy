//! SQLite storage (`app.db` in the app data directory): settings, history and saved palettes.
//!
//! Migrations are numbered SQL scripts applied in order and tracked with `PRAGMA user_version`,
//! each in its own transaction, so a fresh install and an existing database end up identical.

use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

use color_core::palette::{Role, Scheme};
use color_core::simulate::{VisionProfile, VisionType};

/// Oldest history entries beyond this are dropped.
pub const HISTORY_CAP: i64 = 500;

/// Migration N brings `user_version` from N to N + 1.
const MIGRATIONS: &[&str] = &[
    // 1: initial schema (SPEC Phase 7), plus `locked` on palette colours.
    "CREATE TABLE profile (
        id         INTEGER PRIMARY KEY CHECK (id = 1),
        vision     TEXT NOT NULL,
        severity   REAL NOT NULL,
        name_level TEXT NOT NULL
     );
     CREATE TABLE history (
        id        INTEGER PRIMARY KEY AUTOINCREMENT,
        hex       TEXT NOT NULL,
        picked_at INTEGER NOT NULL,
        source    TEXT NOT NULL
     );
     CREATE INDEX history_picked_at ON history (picked_at);
     CREATE TABLE palettes (
        id         INTEGER PRIMARY KEY AUTOINCREMENT,
        name       TEXT NOT NULL,
        created_at INTEGER NOT NULL,
        base_hex   TEXT NOT NULL,
        scheme     TEXT NOT NULL
     );
     CREATE TABLE palette_colors (
        palette_id INTEGER NOT NULL REFERENCES palettes (id) ON DELETE CASCADE,
        position   INTEGER NOT NULL,
        hex        TEXT NOT NULL,
        role       TEXT NOT NULL,
        locked     INTEGER NOT NULL DEFAULT 0,
        PRIMARY KEY (palette_id, position)
     );
     CREATE TABLE settings (
        key   TEXT PRIMARY KEY,
        value TEXT NOT NULL
     );",
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub onboarded: bool,
    pub profile: VisionProfile,
    pub profile_unsure: bool,
    pub hotkey: String,
    pub sample_size: u32,
    pub theme: String,
    pub name_level: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            onboarded: false,
            profile: VisionProfile {
                vision: VisionType::Deutan,
                severity: 0.6,
            },
            profile_unsure: true,
            hotkey: "CommandOrControl+Shift+C".into(),
            sample_size: 5,
            theme: "system".into(),
            name_level: "descriptive".into(),
        }
    }
}

/// Settings plus whether anything was ever saved (so the UI can import older local settings).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredSettings {
    pub settings: Settings,
    pub saved: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: i64,
    pub hex: String,
    /// Unix time in milliseconds.
    pub picked_at: i64,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaletteColorInput {
    pub hex: String,
    pub role: Role,
    #[serde(default)]
    pub locked: bool,
}

/// A palette to save; with `id` it replaces that saved palette.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaletteInput {
    pub id: Option<i64>,
    pub name: String,
    pub base_hex: String,
    pub scheme: Scheme,
    pub colors: Vec<PaletteColorInput>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedPalette {
    pub id: i64,
    pub name: String,
    /// Unix time in milliseconds.
    pub created_at: i64,
    pub base_hex: String,
    pub scheme: Scheme,
    pub colors: Vec<PaletteColorInput>,
}

pub struct Storage {
    conn: Mutex<Connection>,
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn json<T: Serialize>(v: &T) -> String {
    serde_json::to_string(v).expect("serialisable")
}

fn from_json<T: for<'de> Deserialize<'de>>(s: &str) -> Result<T, String> {
    serde_json::from_str(s).map_err(|e| e.to_string())
}

fn err(e: rusqlite::Error) -> String {
    format!("database error: {e}")
}

/// Apply pending migrations. Returns the schema version afterwards.
pub fn migrate(conn: &mut Connection) -> rusqlite::Result<usize> {
    conn.pragma_update(None, "foreign_keys", true)?;
    let mut version: i64 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    while (version as usize) < MIGRATIONS.len() {
        let tx = conn.transaction()?;
        tx.execute_batch(MIGRATIONS[version as usize])?;
        tx.pragma_update(None, "user_version", version + 1)?;
        tx.commit()?;
        version += 1;
    }
    Ok(version as usize)
}

impl Storage {
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let mut conn = Connection::open(path).map_err(err)?;
        migrate(&mut conn).map_err(err)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    #[cfg(test)]
    pub fn in_memory() -> Self {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn).unwrap();
        Self {
            conn: Mutex::new(conn),
        }
    }

    // --- settings ------------------------------------------------------------------------

    pub fn settings(&self) -> Result<StoredSettings, String> {
        let conn = self.conn.lock().unwrap();
        let mut s = Settings::default();
        let profile = conn
            .query_row(
                "SELECT vision, severity, name_level FROM profile WHERE id = 1",
                [],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, f64>(1)?,
                        r.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(err)?;
        let saved = profile.is_some();
        if let Some((vision, severity, name_level)) = profile {
            s.profile = VisionProfile {
                vision: from_json(&vision)?,
                severity: severity as f32,
            };
            s.name_level = name_level;
        }
        let mut stmt = conn
            .prepare("SELECT key, value FROM settings")
            .map_err(err)?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(err)?;
        for row in rows {
            let (key, value) = row.map_err(err)?;
            match key.as_str() {
                "onboarded" => s.onboarded = from_json(&value)?,
                "profile_unsure" => s.profile_unsure = from_json(&value)?,
                "hotkey" => s.hotkey = from_json(&value)?,
                "sample_size" => s.sample_size = from_json(&value)?,
                "theme" => s.theme = from_json(&value)?,
                _ => {} // keys from newer versions are ignored
            }
        }
        Ok(StoredSettings { settings: s, saved })
    }

    pub fn save_settings(&self, s: &Settings) -> Result<(), String> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction().map_err(err)?;
        tx.execute(
            "INSERT INTO profile (id, vision, severity, name_level) VALUES (1, ?1, ?2, ?3)
             ON CONFLICT (id) DO UPDATE SET vision = ?1, severity = ?2, name_level = ?3",
            params![
                json(&s.profile.vision),
                s.profile.severity as f64,
                s.name_level
            ],
        )
        .map_err(err)?;
        let pairs = [
            ("onboarded", json(&s.onboarded)),
            ("profile_unsure", json(&s.profile_unsure)),
            ("hotkey", json(&s.hotkey)),
            ("sample_size", json(&s.sample_size)),
            ("theme", json(&s.theme)),
        ];
        for (key, value) in pairs {
            tx.execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT (key) DO UPDATE SET value = ?2",
                params![key, value],
            )
            .map_err(err)?;
        }
        tx.commit().map_err(err)
    }

    /// Defaults for everything, but keep `onboarded` so the welcome steps don't return.
    pub fn reset_settings(&self) -> Result<Settings, String> {
        let onboarded = self.settings()?.settings.onboarded;
        let s = Settings {
            onboarded,
            ..Settings::default()
        };
        self.save_settings(&s)?;
        Ok(s)
    }

    // --- history -------------------------------------------------------------------------

    pub fn add_history(&self, hex: &str, source: &str) -> Result<HistoryEntry, String> {
        let conn = self.conn.lock().unwrap();
        let picked_at = now_ms();
        conn.execute(
            "INSERT INTO history (hex, picked_at, source) VALUES (?1, ?2, ?3)",
            params![hex, picked_at, source],
        )
        .map_err(err)?;
        let id = conn.last_insert_rowid();
        conn.execute(
            "DELETE FROM history WHERE id NOT IN
               (SELECT id FROM history ORDER BY id DESC LIMIT ?1)",
            params![HISTORY_CAP],
        )
        .map_err(err)?;
        Ok(HistoryEntry {
            id,
            hex: hex.into(),
            picked_at,
            source: source.into(),
        })
    }

    /// Newest first.
    pub fn list_history(&self, limit: i64) -> Result<Vec<HistoryEntry>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, hex, picked_at, source FROM history ORDER BY id DESC LIMIT ?1")
            .map_err(err)?;
        stmt.query_map(params![limit], |r| {
            Ok(HistoryEntry {
                id: r.get(0)?,
                hex: r.get(1)?,
                picked_at: r.get(2)?,
                source: r.get(3)?,
            })
        })
        .map_err(err)?
        .collect::<rusqlite::Result<_>>()
        .map_err(err)
    }

    pub fn delete_history_entry(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM history WHERE id = ?1", params![id])
            .map_err(err)?;
        Ok(())
    }

    pub fn clear_history(&self) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM history", []).map_err(err)?;
        Ok(())
    }

    // --- palettes ------------------------------------------------------------------------

    pub fn save_palette(&self, p: &PaletteInput) -> Result<SavedPalette, String> {
        let name = p.name.trim();
        if name.is_empty() {
            return Err("Give the palette a name.".into());
        }
        if p.colors.is_empty() {
            return Err("A palette needs at least one colour.".into());
        }
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction().map_err(err)?;
        let scheme = json(&p.scheme);
        let (id, created_at) = match p.id {
            Some(id) => {
                let created_at: i64 = tx
                    .query_row(
                        "SELECT created_at FROM palettes WHERE id = ?1",
                        params![id],
                        |r| r.get(0),
                    )
                    .optional()
                    .map_err(err)?
                    .ok_or("That palette no longer exists.")?;
                tx.execute(
                    "UPDATE palettes SET name = ?2, base_hex = ?3, scheme = ?4 WHERE id = ?1",
                    params![id, name, p.base_hex, scheme],
                )
                .map_err(err)?;
                tx.execute(
                    "DELETE FROM palette_colors WHERE palette_id = ?1",
                    params![id],
                )
                .map_err(err)?;
                (id, created_at)
            }
            None => {
                let created_at = now_ms();
                tx.execute(
                    "INSERT INTO palettes (name, created_at, base_hex, scheme)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![name, created_at, p.base_hex, scheme],
                )
                .map_err(err)?;
                (tx.last_insert_rowid(), created_at)
            }
        };
        for (i, c) in p.colors.iter().enumerate() {
            tx.execute(
                "INSERT INTO palette_colors (palette_id, position, hex, role, locked)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, i as i64, c.hex, json(&c.role), c.locked],
            )
            .map_err(err)?;
        }
        tx.commit().map_err(err)?;
        Ok(SavedPalette {
            id,
            name: name.into(),
            created_at,
            base_hex: p.base_hex.clone(),
            scheme: p.scheme,
            colors: p.colors.clone(),
        })
    }

    pub fn rename_palette(&self, id: i64, name: &str) -> Result<(), String> {
        let name = name.trim();
        if name.is_empty() {
            return Err("Give the palette a name.".into());
        }
        let conn = self.conn.lock().unwrap();
        let n = conn
            .execute(
                "UPDATE palettes SET name = ?2 WHERE id = ?1",
                params![id, name],
            )
            .map_err(err)?;
        if n == 0 {
            return Err("That palette no longer exists.".into());
        }
        Ok(())
    }

    pub fn delete_palette(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM palettes WHERE id = ?1", params![id])
            .map_err(err)?;
        Ok(())
    }

    pub fn palette(&self, id: i64) -> Result<SavedPalette, String> {
        self.list_palettes()?
            .into_iter()
            .find(|p| p.id == id)
            .ok_or_else(|| "That palette no longer exists.".into())
    }

    /// Newest first.
    pub fn list_palettes(&self) -> Result<Vec<SavedPalette>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, name, created_at, base_hex, scheme FROM palettes
                 ORDER BY created_at DESC, id DESC",
            )
            .map_err(err)?;
        let heads = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                ))
            })
            .map_err(err)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(err)?;
        let mut colors = conn
            .prepare(
                "SELECT hex, role, locked FROM palette_colors
                 WHERE palette_id = ?1 ORDER BY position",
            )
            .map_err(err)?;
        heads
            .into_iter()
            .map(|(id, name, created_at, base_hex, scheme)| {
                let cols = colors
                    .query_map(params![id], |r| {
                        Ok((
                            r.get::<_, String>(0)?,
                            r.get::<_, String>(1)?,
                            r.get::<_, bool>(2)?,
                        ))
                    })
                    .map_err(err)?
                    .map(|row| {
                        let (hex, role, locked) = row.map_err(err)?;
                        Ok(PaletteColorInput {
                            hex,
                            role: from_json(&role)?,
                            locked,
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                Ok(SavedPalette {
                    id,
                    name,
                    created_at,
                    base_hex,
                    scheme: from_json(&scheme)?,
                    colors: cols,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(name: &str) -> PaletteInput {
        PaletteInput {
            id: None,
            name: name.into(),
            base_hex: "#87CEEB".into(),
            scheme: Scheme::Complementary,
            colors: vec![
                PaletteColorInput {
                    hex: "#F3F9FF".into(),
                    role: Role::Background,
                    locked: false,
                },
                PaletteColorInput {
                    hex: "#87CEEB".into(),
                    role: Role::Primary,
                    locked: true,
                },
            ],
        }
    }

    #[test]
    fn migrates_a_fresh_database_and_is_idempotent() {
        let mut conn = Connection::open_in_memory().unwrap();
        assert_eq!(migrate(&mut conn).unwrap(), MIGRATIONS.len());
        assert_eq!(migrate(&mut conn).unwrap(), MIGRATIONS.len());
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        for t in [
            "history",
            "palette_colors",
            "palettes",
            "profile",
            "settings",
        ] {
            assert!(tables.contains(&t.to_string()), "{t} missing: {tables:?}");
        }
    }

    #[test]
    fn data_survives_reopening_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("app.db");
        {
            let db = Storage::open(&path).unwrap();
            db.add_history("#87CEEB", "picker").unwrap();
            db.save_palette(&sample("Sky")).unwrap();
            let s = Settings {
                onboarded: true,
                theme: "dark".into(),
                ..Settings::default()
            };
            db.save_settings(&s).unwrap();
        }
        let db = Storage::open(&path).unwrap();
        assert_eq!(db.list_history(10).unwrap()[0].hex, "#87CEEB");
        assert_eq!(db.list_palettes().unwrap()[0].name, "Sky");
        let stored = db.settings().unwrap();
        assert!(stored.saved && stored.settings.onboarded);
        assert_eq!(stored.settings.theme, "dark");
    }

    #[test]
    fn settings_default_until_saved_and_reset_keeps_onboarding() {
        let db = Storage::in_memory();
        let stored = db.settings().unwrap();
        assert!(!stored.saved);
        assert_eq!(stored.settings, Settings::default());

        let s = Settings {
            onboarded: true,
            profile: VisionProfile {
                vision: VisionType::Normal,
                severity: 0.0,
            },
            profile_unsure: false,
            hotkey: "Alt+P".into(),
            sample_size: 3,
            theme: "light".into(),
            name_level: "basic".into(),
        };
        db.save_settings(&s).unwrap();
        assert_eq!(db.settings().unwrap().settings, s);

        let reset = db.reset_settings().unwrap();
        assert_eq!(
            reset,
            Settings {
                onboarded: true,
                ..Settings::default()
            }
        );
        assert_eq!(db.settings().unwrap().settings, reset);
    }

    #[test]
    fn history_is_newest_first_capped_and_deletable() {
        let db = Storage::in_memory();
        for i in 0..(HISTORY_CAP + 5) {
            db.add_history(&format!("#{i:06X}"), "picker").unwrap();
        }
        let all = db.list_history(10_000).unwrap();
        assert_eq!(all.len() as i64, HISTORY_CAP);
        assert_eq!(all[0].hex, format!("#{:06X}", HISTORY_CAP + 4));

        db.delete_history_entry(all[0].id).unwrap();
        assert_eq!(db.list_history(1).unwrap()[0].id, all[1].id);
        db.clear_history().unwrap();
        assert!(db.list_history(10).unwrap().is_empty());
    }

    #[test]
    fn palettes_save_update_rename_and_delete() {
        let db = Storage::in_memory();
        let saved = db.save_palette(&sample("Sky")).unwrap();
        assert_eq!(db.palette(saved.id).unwrap().colors, sample("Sky").colors);

        let mut edit = sample("Sky v2");
        edit.id = Some(saved.id);
        edit.colors.pop();
        let updated = db.save_palette(&edit).unwrap();
        assert_eq!(updated.created_at, saved.created_at);
        assert_eq!(db.palette(saved.id).unwrap().colors.len(), 1);

        db.rename_palette(saved.id, "  Evening sky ").unwrap();
        assert_eq!(db.palette(saved.id).unwrap().name, "Evening sky");
        assert!(db.rename_palette(saved.id, "   ").is_err());

        db.save_palette(&sample("Other")).unwrap();
        assert_eq!(db.list_palettes().unwrap().len(), 2);
        db.delete_palette(saved.id).unwrap();
        let left = db.list_palettes().unwrap();
        assert_eq!(left.len(), 1);
        assert_eq!(left[0].name, "Other");
        assert!(db.palette(saved.id).is_err());
    }

    #[test]
    fn rejects_unnamed_or_empty_palettes() {
        let db = Storage::in_memory();
        assert!(db.save_palette(&sample("  ")).is_err());
        let mut empty = sample("Empty");
        empty.colors.clear();
        assert!(db.save_palette(&empty).is_err());
    }
}
