//! # Languatage
//! This is a tool for calculate the percentage of languages used in a directory.
//!
//! ## Usage
//!
//! ```rust
//! use languatage::{get_stat, LanguageStat};
//!
//! let stat: std::io::Result<Vec<LanguageStat>> = get_stat(".");
//! ```

pub mod config;

pub use crate::config::Config;
use std::{
    borrow::Cow,
    fs::{self, DirEntry},
    path::{Path, MAIN_SEPARATOR},
};

#[derive(Debug, Clone, PartialEq)]
pub struct LanguageStat {
    pub lang: String,
    pub size: u64,
    pub percentage: f64,
}

/// Returns language usage statistics.
/// ```rust
/// use languatage::{get_stat, LanguageStat};
///
/// let stat: std::io::Result<Vec<LanguageStat>> = get_stat(".");
/// ```
pub fn get_stat<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<LanguageStat>> {
    let config = Config::default();
    get_stat_with_config(path, &config)
}

/// Returns language usage statistics based on specified config.
/// ```rust
/// use languatage::{get_stat_with_config, Config, LanguageStat};
///
/// let config: Config = Config::default();
/// let stat: std::io::Result<Vec<LanguageStat>> = get_stat_with_config(".", &config);
/// ```
pub fn get_stat_with_config<P: AsRef<Path>>(
    path: P,
    config: &Config,
) -> std::io::Result<Vec<LanguageStat>> {
    let sizes = get_size(path, config)?;
    let mut sizes = sizes
        .into_iter()
        .filter(|(_, s)| *s != 0)
        .collect::<Vec<_>>();
    sizes.sort_by(|a, b| b.1.cmp(&a.1));

    let total_size: u64 = sizes.iter().map(|v| v.1).sum();

    let result = sizes
        .iter()
        .map(|v| LanguageStat {
            lang: v.0.clone(),
            size: v.1,
            percentage: v.1 as f64 / total_size as f64 * 100.0,
        })
        .collect();

    Ok(result)
}

fn get_size<P: AsRef<Path>>(path: P, config: &Config) -> std::io::Result<Vec<(String, u64)>> {
    let common_ignores = &config.common.ignore;

    let result = config
        .language
        .iter()
        .filter_map(|language| {
            // concat common_ignores and lang_ignores
            let ignores: Vec<_> = common_ignores
                .iter()
                .chain(language.ignore.iter())
                .collect();

            let entries = &get_dir_entries(&path, &ignores, &language.ext)?;

            let size: u64 = entries
                .iter()
                .filter_map(|v| Some(v.metadata().ok()?.len()))
                .sum();

            Some((language.lang.clone(), size))
        })
        .collect();

    Ok(result)
}

/// Returns all files under the given path that match the common config
fn get_dir_entries<
    'a,
    P: AsRef<Path>,
    S: Into<Cow<'a, str>> + std::fmt::Display,
    X: Into<Cow<'a, str>> + std::fmt::Display,
>(
    path: P,
    ignores: &[S],
    exts: &[X],
) -> Option<Vec<DirEntry>> {
    let path = path.as_ref().to_str()?;

    let is_dot_dir = path != "." && path.split(&['/', '\\'][..]).last()?.starts_with('.');

    if is_dot_dir {
        return None;
    }

    let read_dir = match fs::read_dir(path) {
        Ok(read_dir) => read_dir,
        Err(_) => return None,
    };

    let result = read_dir
        .into_iter()
        .filter_map(|entry| -> Option<Vec<DirEntry>> {
            let entry = entry.ok()?;

            let entry_path = entry.path();
            let entry_path = entry_path.to_string_lossy();

            let is_ignored = ignores.iter().any(|ignore| {
                entry_path.contains(&format!("{}{}{}", MAIN_SEPARATOR, ignore, MAIN_SEPARATOR))
            });

            if is_ignored {
                return None;
            };

            if entry.metadata().ok()?.is_dir() {
                return get_dir_entries(entry.path(), ignores, exts);
            };

            let is_correct_ext = exts
                .iter()
                .any(|ext| entry_path.ends_with(&format!(".{}", ext)));

            if is_correct_ext {
                Some(vec![entry])
            } else {
                None
            }
        })
        .flatten()
        .collect();

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_stat() {
        let stat = get_stat(".").unwrap();

        assert_eq!(stat[0].lang, "Rust".to_string());
        assert_eq!(stat[0].percentage, 100.0);
    }

    #[test]
    fn test_get_stat_with_config() {
        let config = Config::default();
        let stat = get_stat_with_config(".", &config).unwrap();

        assert_eq!(stat[0].lang, "Rust".to_string());
        assert_eq!(stat[0].percentage, 100.0);
        assert_eq!(stat.len(), 1);
    }

    #[test]
    fn test_get_dir_entries() {
        let config = Config::default();
        let common_ignores = &config.common.ignore;
        let lang_ignores = &config.language[0].ignore;
        let ignores: Vec<_> = common_ignores.iter().chain(lang_ignores.iter()).collect();

        assert_eq!(
            get_dir_entries(".", common_ignores, &config.language[0].ext)
                .unwrap()
                .iter()
                .any(|entry| entry
                    .path()
                    .to_string_lossy()
                    .contains(&format!("{}.git{}", MAIN_SEPARATOR, MAIN_SEPARATOR))),
            false
        );

        assert_eq!(
            get_dir_entries(".", &ignores, &config.language[0].ext)
                .unwrap()
                .iter()
                .any(|entry| entry
                    .path()
                    .to_string_lossy()
                    .contains(&format!("{}.git{}", MAIN_SEPARATOR, MAIN_SEPARATOR))),
            false
        );
    }
}
