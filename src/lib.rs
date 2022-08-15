use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{self, Path},
};

const CONFIG: &str = include_str!("config.yaml");

/// Get percentage of language under the passed path
pub fn exec<P: AsRef<Path>>(path: P) -> Vec<LanguageStat> {
    let configs = parse_config();
    let files = get_files(path);
    get_stats(&configs, &files)
}

#[derive(Debug, Clone, PartialEq)]
pub struct LanguageStat {
    lang: String,
    percentage: f64,
}

#[derive(Debug, Clone, PartialEq)]
struct LanguageSize {
    lang: String,
    size: usize,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct ConfigItem {
    lang: String,
    ext: Vec<String>,
    ignore: Vec<String>,
}

fn parse_config() -> Vec<ConfigItem> {
    let config: Vec<ConfigItem> = serde_yaml::from_str(CONFIG).unwrap();
    config
}

/// Returns all files under the passed path
fn get_files<P: AsRef<Path>>(path: P) -> Vec<String> {
    let dir = fs::read_dir(path).unwrap();
    let paths: Vec<String> = dir
        .flat_map(|v| {
            let file = v.unwrap();
            if file.metadata().unwrap().is_dir() {
                get_files(file.path())
            } else {
                vec![file.path().to_string_lossy().to_string()]
            }
        })
        .collect();

    paths
}

fn get_stats(configs: &[ConfigItem], files: &[String]) -> Vec<LanguageStat> {
    let stats: Vec<LanguageSize> = configs
        .iter()
        .map(|config| {
            let ConfigItem { lang, ext, ignore } = config;
            let size: usize = files
                .iter()
                .filter(|&file| {
                    ignore.iter().all(|ignore| {
                        !file.starts_with(&format!(
                            ".{}{}{}",
                            path::MAIN_SEPARATOR,
                            ignore,
                            path::MAIN_SEPARATOR
                        ))
                    })
                })
                .filter(|&file| ext.iter().any(|ext| file.ends_with(&format!(".{}", ext))))
                .map(|path| fs::read(path).unwrap().len())
                .sum();

            LanguageSize {
                lang: lang.to_string(),
                size,
            }
        })
        .collect();

    let all_size: usize = stats.iter().map(|v| v.size).sum();

    let stats: Vec<LanguageStat> = stats
        .into_iter()
        .map(|v| LanguageStat {
            lang: v.lang,
            percentage: v.size as f64 / all_size as f64 * 100.0,
        })
        .filter(|v| v.percentage != 0.0)
        .collect();

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_config() {
        let config = parse_config();
        assert_eq!(
            config[0],
            ConfigItem {
                lang: "rust".into(),
                ext: vec!["rs".into()],
                ignore: vec!["target".into()]
            }
        )
    }

    #[test]
    fn test() {
        assert_eq!(
            exec("."),
            vec![LanguageStat {
                lang: "rust".into(),
                percentage: 100.0
            }]
        );
    }
}
