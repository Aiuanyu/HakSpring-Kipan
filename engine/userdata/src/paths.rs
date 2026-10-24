//! Where the four stores' files live — the one piece of user data the
//! platform decides (roadmap U5).

use std::path::{Path, PathBuf};

/// Every platform's file names for the four stores.
pub const FREQUENCY_FILE: &str = "user_frequency.db";
pub const ASSOCIATION_FILE: &str = "user_association.db";
pub const CUSTOM_DICTIONARY_FILE: &str = "custom_dictionary.db";
pub const LEARNED_PHRASES_FILE: &str = "learned_phrases.db";

/// Where each store's file lives. The platform decides (roadmap U5): one
/// directory everywhere except Android, whose `user_association.db` has
/// always lived in `filesDir` beside the others' `databases/`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserDataPaths {
    pub frequency: PathBuf,
    pub association: PathBuf,
    pub custom_dictionary: PathBuf,
    pub learned_phrases: PathBuf,
}

/// Hakfa fork: the part of a learning file's name that says whose it is. Lect 0 (a platform that
/// sends none) and 1 (四縣) keep the original names, so what was learned before lects existed is
/// 四縣's without moving anything; 2 (海陸) and the rest get their own files. The custom
/// dictionary is the user's own explicit entries and stays one file for every lect.
pub fn lect_file_suffix(lect: u32) -> Option<String> {
    match lect {
        0 | 1 => None,
        2 => Some("hoiliug".to_owned()),
        other => Some(format!("lect{other}")),
    }
}

/// `user_frequency.db` → `user_frequency.hoiliug.db`.
fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let stem = path.file_stem().and_then(|stem| stem.to_str()).unwrap_or("user_data");
    let name = match path.extension().and_then(|extension| extension.to_str()) {
        Some(extension) => format!("{stem}.{suffix}.{extension}"),
        None => format!("{stem}.{suffix}"),
    };
    path.with_file_name(name)
}

impl UserDataPaths {
    /// The three learning files (frequency, association, learned phrases) for `lect`;
    /// the custom dictionary path is unchanged. Each keeps its own directory (Android's
    /// association file lives elsewhere).
    pub fn for_lect(&self, lect: u32) -> Self {
        let Some(suffix) = lect_file_suffix(lect) else {
            return self.clone();
        };
        Self {
            frequency: with_suffix(&self.frequency, &suffix),
            association: with_suffix(&self.association, &suffix),
            custom_dictionary: self.custom_dictionary.clone(),
            learned_phrases: with_suffix(&self.learned_phrases, &suffix),
        }
    }

    /// All four files in `directory`, under their shared names.
    pub fn in_directory(directory: &Path) -> Self {
        Self {
            frequency: directory.join(FREQUENCY_FILE),
            association: directory.join(ASSOCIATION_FILE),
            custom_dictionary: directory.join(CUSTOM_DICTIONARY_FILE),
            learned_phrases: directory.join(LEARNED_PHRASES_FILE),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lect_zero_and_one_keep_the_original_names() {
        let paths = UserDataPaths::in_directory(Path::new("/data"));
        assert_eq!(paths.for_lect(0), paths);
        assert_eq!(paths.for_lect(1), paths);
    }

    #[test]
    fn another_lect_renames_the_three_learning_files_and_keeps_the_custom_dictionary() {
        let mut paths = UserDataPaths::in_directory(Path::new("/data"));
        paths.association = PathBuf::from("/files/user_association.db");
        let hailu = paths.for_lect(2);
        assert_eq!(hailu.frequency, PathBuf::from("/data/user_frequency.hoiliug.db"));
        assert_eq!(hailu.learned_phrases, PathBuf::from("/data/learned_phrases.hoiliug.db"));
        assert_eq!(hailu.association, PathBuf::from("/files/user_association.hoiliug.db"));
        assert_eq!(hailu.custom_dictionary, paths.custom_dictionary);
        assert_eq!(paths.for_lect(5).frequency, PathBuf::from("/data/user_frequency.lect5.db"));
    }
}
