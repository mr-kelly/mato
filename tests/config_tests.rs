use mato::config::Config;

#[test]
fn test_default_config() {
    let config = Config::default();
    assert_eq!(config.emulator, "vt100");
}

#[test]
fn test_config_serialization() {
    let config = Config {
        emulator: "vte".to_string(),
    };
    
    let toml = toml::to_string(&config).unwrap();
    assert!(toml.contains("vte"));
    
    let deserialized: Config = toml::from_str(&toml).unwrap();
    assert_eq!(deserialized.emulator, "vte");
}

#[test]
fn test_config_load_default() {
    // Loading non-existent config should return default
    let config = Config::load();
    assert_eq!(config.emulator, "vt100");
}
