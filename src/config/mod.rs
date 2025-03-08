mod defaults;
use crossterm::event::KeyCode;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub keybindings: KeybindingsConfig,
    #[serde(default = "defaults::config_path")]
    pub config_path: Option<PathBuf>,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            ui: UiConfig::default(),
            keybindings: KeybindingsConfig::default(),
            config_path: defaults::config_path(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiConfig {
    #[serde(default = "defaults::ui::signal_list_width")]
    pub signal_list_width: u16,
    #[serde(default = "defaults::ui::marker_color_primary")]
    pub marker_color_primary: String,
    #[serde(default = "defaults::ui::marker_color_secondary")]
    pub marker_color_secondary: String,
    #[serde(default = "defaults::ui::drag_color")]
    pub drag_color: String,
}

impl Default for UiConfig {
    fn default() -> Self {
        UiConfig {
            signal_list_width: defaults::ui::signal_list_width(),
            marker_color_primary: defaults::ui::marker_color_primary(),
            marker_color_secondary: defaults::ui::marker_color_secondary(),
            drag_color: defaults::ui::drag_color(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeybindingsConfig {
    #[serde(default = "defaults::keys::enter_command_mode")]
    pub enter_command_mode: KeyCode,

    #[serde(default = "defaults::keys::up")]
    pub up: KeyCode,

    #[serde(default = "defaults::keys::down")]
    pub down: KeyCode,

    #[serde(default = "defaults::keys::left")]
    pub left: KeyCode,

    #[serde(default = "defaults::keys::right")]
    pub right: KeyCode,

    #[serde(default = "defaults::keys::zoom_in")]
    pub zoom_in: KeyCode,

    #[serde(default = "defaults::keys::zoom_out")]
    pub zoom_out: KeyCode,

    #[serde(default = "defaults::keys::zoom_full")]
    pub zoom_full: KeyCode,

    #[serde(default = "defaults::keys::delete_primary_marker")]
    pub delete_primary_marker: KeyCode,

    #[serde(default = "defaults::keys::delete_secondary_marker")]
    pub delete_secondary_marker: KeyCode,

    #[serde(default = "defaults::keys::enter_normal_mode")]
    pub enter_normal_mode: KeyCode,

    #[serde(default = "defaults::keys::execute_command")]
    pub execute_command: KeyCode,
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        Self {
            enter_command_mode: defaults::keys::enter_command_mode(),
            up: defaults::keys::up(),
            down: defaults::keys::down(),
            left: defaults::keys::left(),
            right: defaults::keys::right(),
            zoom_in: defaults::keys::zoom_in(),
            zoom_out: defaults::keys::zoom_out(),
            zoom_full: defaults::keys::zoom_full(),
            delete_primary_marker: defaults::keys::delete_primary_marker(),
            delete_secondary_marker: defaults::keys::delete_secondary_marker(),
            enter_normal_mode: defaults::keys::enter_normal_mode(),
            execute_command: defaults::keys::execute_command(),
        }
    }
}

lazy_static! {
    pub static ref CONFIG: RwLock<AppConfig> = RwLock::new(AppConfig::default());
}

/// Loads application configuration from a file.
///
/// This function handles loading application configuration with the following priorities:
/// 1. From a specified override path if provided
/// 2. From the default config location (platform-specific user config directory following XDG Base
///    Directory Specification)
/// 3. Use default configuration values if no config file exists
///
/// The loaded configuration is stored in the global `CONFIG` static for application-wide access.
///
/// # Arguments
/// * `path_override` - Optional path to a configuration file that overrides the default location
///
/// # Returns
/// * `Ok(AppConfig)` - Successfully loaded or created configuration
/// * `Err(String)` - Error message if loading the configuration failed
pub fn load_config(path_override: Option<String>) -> Result<AppConfig, String> {
    // Check for path override first
    if let Some(override_path) = path_override {
        let path = PathBuf::from(&override_path);
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match toml::from_str::<AppConfig>(&content) {
                    Ok(mut loaded_config) => {
                        // Store the path in the config
                        loaded_config.config_path = Some(path);

                        let mut global_config = CONFIG.write().unwrap();
                        *global_config = loaded_config.clone();
                        return Ok(loaded_config);
                    }
                    Err(e) => {
                        return Err(format!("Error parsing override config file: {}", e));
                    }
                },
                Err(e) => {
                    return Err(format!("Error reading override config file: {}", e));
                }
            }
        } else {
            return Err(format!(
                "Override config path does not exist: {}",
                path.display()
            ));
        }
    }

    // Try to load from default config path if it exists
    if let Some(config_path) = defaults::config_path() {
        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => match toml::from_str::<AppConfig>(&content) {
                    Ok(mut loaded_config) => {
                        // Store the path in the config
                        loaded_config.config_path = Some(config_path);

                        let mut global_config = CONFIG.write().unwrap();
                        *global_config = loaded_config.clone();
                        return Ok(loaded_config);
                    }
                    Err(e) => {
                        return Err(format!("Error parsing config file: {}", e));
                    }
                },
                Err(e) => {
                    return Err(format!("Error reading config file: {}", e));
                }
            }
        }
    }

    // Use default configuration if no config file was found or provided
    Ok(load_default_config())
}

// Build an AppConfig::default() and set it to the global CONFIG variable. This is public so that
// unit tests can access it.
pub fn load_default_config() -> AppConfig {
    let default_config = AppConfig::default();
    let mut global_config = CONFIG.write().unwrap();
    *global_config = default_config.clone();
    default_config
}

pub fn read_config() -> AppConfig {
    CONFIG.read().unwrap().clone()
}

#[cfg(test)]
mod tests {
    use crate::config::{load_config, AppConfig, CONFIG};
    use crossterm::event::KeyCode;
    use std::fs;
    use tempfile::NamedTempFile;

    fn setup() {
        reset_config();
    }

    fn teardown() {
        reset_config();
    }

    // Reset the global CONFIG to default after each test
    fn reset_config() {
        let default_config = AppConfig::default();
        let mut global_config = CONFIG.write().unwrap();
        *global_config = default_config;
    }

    #[test]
    fn test_load_no_file_uses_default_config() {
        setup();
        let config = load_config(None).unwrap();
        assert_eq!(config.ui.signal_list_width, 20);
        assert_eq!(config.keybindings.zoom_in, KeyCode::Char('+'));
        teardown();
    }

    #[test]
    fn test_load_custom_config_uses_custom_values() {
        setup();
        let temp_file = NamedTempFile::new().unwrap();
        let custom_config = r#"
        [ui]
        signal_list_width = 30

        [keybindings]
        zoom_in = { Char = "=" }
        "#;
        fs::write(&temp_file, custom_config).unwrap();

        let config = load_config(Some(temp_file.path().to_str().unwrap().to_string())).unwrap();

        // Custom values should be used
        assert_eq!(config.ui.signal_list_width, 30);
        assert_eq!(config.keybindings.zoom_in, KeyCode::Char('='));

        // Other values should be defaults
        assert_eq!(config.keybindings.zoom_out, KeyCode::Char('-'));

        teardown();
    }

    #[test]
    fn test_invalid_config_loading_returns_err() {
        setup();

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "this is not valid TOML").unwrap();
        assert!(load_config(Some(temp_file.path().to_str().unwrap().to_string())).is_err());

        teardown();
    }
}
