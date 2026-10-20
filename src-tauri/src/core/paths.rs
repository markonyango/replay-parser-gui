use std::path::{Path, PathBuf};

const GAME_DIR: &str = "Dawn of War II - Retribution";

/// Steam app id of Dawn of War II - Retribution, used for the Proton prefix folder.
#[cfg(target_os = "linux")]
const STEAM_APP_ID: &str = "56400";

/// Returns the candidate `.../My Games/Dawn of War II - Retribution` directories,
/// in priority order. The native documents directory is always last, so Windows
/// behaves exactly as before.
pub fn candidate_game_dirs(document_dir: &Path, home_dir: Option<&Path>) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    #[cfg(target_os = "linux")]
    if let Some(home) = home_dir {
        for library in steam_libraries(home) {
            candidates.push(proton_documents_dir(&library).join("My Games").join(GAME_DIR));
        }
    }
    #[cfg(not(target_os = "linux"))]
    let _ = home_dir;

    candidates.push(document_dir.join("My Games").join(GAME_DIR));
    candidates
}

/// Picks the first candidate that contains a `Logfiles` directory, falling back to
/// the last candidate (native documents dir) so the caller's not-found errors stay meaningful.
pub fn resolve_game_dir(document_dir: &Path, home_dir: Option<&Path>) -> PathBuf {
    let mut candidates = candidate_game_dirs(document_dir, home_dir);
    let fallback = candidates.pop().expect("candidates always has the native entry");

    candidates
        .into_iter()
        .chain(std::iter::once(fallback.clone()))
        .find(|dir| dir.join("Logfiles").is_dir() || dir.join("Playback").is_dir())
        .unwrap_or(fallback)
}

#[cfg(target_os = "linux")]
fn proton_documents_dir(library: &Path) -> PathBuf {
    library
        .join("steamapps/compatdata")
        .join(STEAM_APP_ID)
        .join("pfx/drive_c/users/steamuser/Documents")
}

/// All Steam library folders found for the known Steam install locations.
#[cfg(target_os = "linux")]
fn steam_libraries(home: &Path) -> Vec<PathBuf> {
    let roots = [
        home.join(".local/share/Steam"),
        home.join(".steam/steam"),
        home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
    ];

    let mut libraries: Vec<PathBuf> = Vec::new();
    for root in roots {
        if !root.is_dir() {
            continue;
        }
        // Resolve symlinks (~/.steam/steam usually points at ~/.local/share/Steam).
        let root = root.canonicalize().unwrap_or(root);
        let vdf = root.join("steamapps/libraryfolders.vdf");
        let mut found = std::fs::read_to_string(&vdf)
            .map(|content| parse_library_paths(&content))
            .unwrap_or_default();
        found.push(root);

        for library in found {
            if !libraries.contains(&library) {
                libraries.push(library);
            }
        }
    }
    libraries
}

/// Extracts the `"path"` entries from a Steam `libraryfolders.vdf`.
#[cfg(target_os = "linux")]
fn parse_library_paths(vdf: &str) -> Vec<PathBuf> {
    lazy_static::lazy_static! {
        static ref PATH_RE: regex::Regex = regex::Regex::new(r#""path"\s+"([^"]+)""#).unwrap();
    }
    PATH_RE
        .captures_iter(vdf)
        .map(|c| PathBuf::from(c[1].replace("\\\\", "\\")))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn falls_back_to_native_documents_dir() {
        let tmp = std::env::temp_dir().join("eslreporter-paths-fallback");
        let resolved = resolve_game_dir(&tmp, Some(&tmp));
        assert_eq!(resolved, tmp.join("My Games").join(GAME_DIR));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn parses_library_folders_vdf() {
        let vdf = r#"
"libraryfolders"
{
    "0" { "path" "/home/u/.local/share/Steam" "label" "" }
    "1" { "path" "/mnt/games/SteamLibrary" }
}"#;
        assert_eq!(
            parse_library_paths(vdf),
            vec![
                PathBuf::from("/home/u/.local/share/Steam"),
                PathBuf::from("/mnt/games/SteamLibrary")
            ]
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn resolves_proton_prefix_in_secondary_library() {
        let tmp = std::env::temp_dir().join("eslreporter-paths-proton");
        let _ = std::fs::remove_dir_all(&tmp);
        let home = tmp.join("home");
        let library = tmp.join("SteamLibrary");

        std::fs::create_dir_all(home.join(".local/share/Steam/steamapps")).unwrap();
        std::fs::write(
            home.join(".local/share/Steam/steamapps/libraryfolders.vdf"),
            format!("\"libraryfolders\" {{ \"1\" {{ \"path\" \"{}\" }} }}", library.display()),
        )
        .unwrap();
        let game_dir = proton_documents_dir(&library).join("My Games").join(GAME_DIR);
        std::fs::create_dir_all(game_dir.join("Logfiles")).unwrap();

        let resolved = resolve_game_dir(&home.join("Documents"), Some(&home));
        assert_eq!(resolved, game_dir);

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
