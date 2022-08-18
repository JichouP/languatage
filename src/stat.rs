use crate::config::Config;
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

pub fn get_stat<P: AsRef<Path>>(path: P, config: &Config) -> Vec<LanguageStat> {
    let sizes = get_size(path, config);
    let total_size: u64 = sizes.iter().map(|v| v.1).sum();

    sizes
        .iter()
        .map(|v| LanguageStat {
            lang: v.0.clone(),
            size: v.1,
            percentage: v.1 as f64 / total_size as f64 * 100.0,
        })
        .collect()
}

fn get_size<P: AsRef<Path>>(path: P, config: &Config) -> Vec<(String, u64)> {
    let common_ignores = &config.common.ignore;

    config
        .language
        .iter()
        .map(|language| {
            // concat common_ignores and lang_ignores
            let ignores: Vec<_> = common_ignores
                .iter()
                .chain(language.ignore.iter())
                .collect();

            let entries = &get_dir_entries(&path, &ignores, &language.ext);

            let size: u64 = entries.iter().map(|v| v.metadata().unwrap().len()).sum();

            (language.lang.clone(), size)
        })
        .collect()
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
) -> Vec<DirEntry> {
    fs::read_dir(path)
        .unwrap()
        .into_iter()
        .flat_map(|entry| -> Vec<DirEntry> {
            let entry = entry.unwrap();

            let entry_path = entry.path();
            let entry_path = entry_path.to_string_lossy();

            let is_ignored = ignores.iter().any(|ignore| {
                entry_path.contains(&format!("{}{}{}", MAIN_SEPARATOR, ignore, MAIN_SEPARATOR))
            });

            if is_ignored {
                vec![]
            } else if entry.metadata().unwrap().is_dir() {
                get_dir_entries(entry.path(), ignores, exts)
            } else {
                let is_correct_ext = exts
                    .iter()
                    .any(|ext| entry_path.ends_with(&format!(".{}", ext)));
                if is_correct_ext {
                    vec![entry]
                } else {
                    vec![]
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn get_stat_test() {
        let config = Config::default();
        let stat = get_stat(".", &config);
        assert_eq!(stat[0].lang, "rust".to_string());
        assert_eq!(stat[0].percentage, 100.0);
        assert_eq!(stat.len(), config.language.len());
    }

    #[test]
    fn get_dir_entries_test() {
        let config = Config::default();
        let common_ignores = &config.common.ignore;
        let lang_ignores = &config.language[0].ignore;
        let ignores: Vec<_> = common_ignores.iter().chain(lang_ignores.iter()).collect();

        assert_eq!(
            get_dir_entries(".", common_ignores, &config.language[0].ext)
                .iter()
                .any(|entry| entry
                    .path()
                    .to_string_lossy()
                    .contains(&format!("{}.git{}", MAIN_SEPARATOR, MAIN_SEPARATOR))),
            false
        );

        assert_eq!(
            get_dir_entries(".", lang_ignores, &config.language[0].ext)
                .iter()
                .any(|entry| entry
                    .path()
                    .to_string_lossy()
                    .contains(&format!("{}target{}", MAIN_SEPARATOR, MAIN_SEPARATOR))),
            false
        );

        assert_eq!(
            get_dir_entries(".", &ignores, &config.language[0].ext)
                .iter()
                .any(|entry| entry
                    .path()
                    .to_string_lossy()
                    .contains(&format!("{}.git{}", MAIN_SEPARATOR, MAIN_SEPARATOR))),
            false
        );
    }
}
