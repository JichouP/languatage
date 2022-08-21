use serde::Deserialize;

const DEFAULT_CONFIG: &str = include_str!("config.yaml");

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Config {
    pub language: Vec<LanguageConfigItem>,
    pub common: CommonConfig,
}

impl Config {
    pub fn new(language: Vec<LanguageConfigItem>, common: CommonConfig) -> Self {
        Self { language, common }
    }

    pub fn default() -> Self {
        serde_yaml::from_str(DEFAULT_CONFIG).expect("src/config.yaml is invalid")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LanguageConfigItem {
    pub lang: String,
    pub ext: Vec<String>,
    pub ignore: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CommonConfig {
    pub ignore: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_default_test() {
        let config = Config::default();
        assert_eq!(
            config.language[0],
            LanguageConfigItem {
                lang: "rust".into(),
                ext: vec!["rs".into()],
                ignore: vec![]
            }
        )
    }
}
