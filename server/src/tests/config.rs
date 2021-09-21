use crate::config::Config;

/// Ensure `config.toml` loads without issues.
#[test]
fn load_config() {
    load().expect("failed to load config from file");
}
