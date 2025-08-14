use riglr_core::util::{init_env_from_file, get_env_vars, validate_required_env, get_required_env, get_env_or_default, EnvError};
use std::{env, fs, io::Write};

#[test]
fn init_env_from_file_nonexistent_is_ok() {
    // A random path that should not exist
    let res = init_env_from_file("./this_file_should_not_exist.env");
    assert!(res.is_ok());
}

#[test]
fn init_env_from_file_valid_and_malformed() {
    // Create a temp valid .env
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "FOO=bar").unwrap();
    let path = file.path().to_str().unwrap().to_string();

    // Should load fine
    init_env_from_file(&path).unwrap();
    assert_eq!(env::var("FOO").unwrap(), "bar");

    // Create malformed file (no key)
    let mut bad = tempfile::NamedTempFile::new().unwrap();
    writeln!(bad, "=novalue").unwrap();
    let bad_path = bad.path().to_str().unwrap().to_string();

    // Depending on dotenv parsing, this may error; our function maps it to io::Error
    let res = init_env_from_file(&bad_path);
    assert!(res.is_err());
}

#[test]
fn validate_and_get_helpers() {
    env::remove_var("A");
    env::set_var("B", "b");

    // get_env_or_default
    assert_eq!(get_env_or_default("A", "def"), "def");

    // get_required_env missing
    let err = get_required_env("A").unwrap_err();
    match err {
        EnvError::MissingRequired(k) => assert_eq!(k, "A"),
        _ => panic!("expected MissingRequired"),
    }

    // validate_required_env
    let res = validate_required_env(&["A", "B"]);
    assert!(res.is_err());

    // get_env_vars
    let map = get_env_vars(&["A", "B"]);
    assert!(map.get("A").is_none());
    assert_eq!(map.get("B").map(String::as_str), Some("b"));

    env::remove_var("B");
}
