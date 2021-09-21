use crate::config;

/// Ensure `config.toml` loads without issues.
#[test]
fn load_config() {
    config::load().expect("failed to load config from file");
}
