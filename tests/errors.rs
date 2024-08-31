// tests/errors.rs

use clap::{Arg, Command};
use std::error::Error;
use std::ffi::OsString;
use std::io;
use strum::ParseError as StrumParseError;
use toml::de::Error as TomlDeError;
use toml::ser::Error as TomlSerError;

use thag_rs::errors::ThagError;

// Set environment variables before running tests
fn set_up() {
    std::env::set_var("TEST_ENV", "1");
    std::env::set_var("VISUAL", "cat");
    std::env::set_var("EDITOR", "cat");
}

#[test]
fn test_io_error() {
    set_up();
    let io_err = io::Error::new(io::ErrorKind::Other, "I/O error occurred");
    let build_run_err: ThagError = io_err.into();
    match build_run_err {
        ThagError::Io(_) => (),
        _ => panic!("Expected ThagError::Io variant"),
    }
}

#[test]
fn test_clap_error() {
    set_up();
    let clap_err = Command::new("test")
        .arg(Arg::new("arg").required(true))
        .try_get_matches_from(vec!["test"])
        .unwrap_err();
    let build_run_err: ThagError = clap_err.into();
    match build_run_err {
        ThagError::ClapError(_) => (),
        _ => panic!("Expected ThagError::ClapError variant"),
    }
}

#[test]
fn test_strum_parse_error() {
    set_up();
    let strum_err = StrumParseError::VariantNotFound;
    let build_run_err: ThagError = strum_err.into();
    match build_run_err {
        ThagError::StrumParse(_) => (),
        _ => panic!("Expected ThagError::StrumParse variant"),
    }
}

#[test]
fn test_toml_de_error() {
    set_up();
    let toml_str = "invalid = toml";
    let toml_err: Result<toml::Value, TomlDeError> = toml::from_str(toml_str);
    let build_run_err: ThagError = toml_err.unwrap_err().into();
    match build_run_err {
        ThagError::TomlDe(_) => (),
        _ => panic!("Expected ThagError::TomlDe variant"),
    }
}

#[test]
fn test_toml_ser_error() {
    set_up();
    let value = toml::Value::String("test".to_string());
    let toml_err: Result<String, TomlSerError> = toml::to_string(&value);
    let build_run_err: ThagError = toml_err.unwrap_err().into();
    match build_run_err {
        ThagError::TomlSer(_) => (),
        _ => panic!("Expected ThagError::TomlSer variant"),
    }
}

#[test]
fn test_from_string() {
    set_up();
    let error_message = String::from("This is a string error");
    let build_run_err: ThagError = error_message.into();
    match build_run_err {
        ThagError::FromStr(_) => (),
        _ => panic!("Expected ThagError::FromStr variant"),
    }
}

#[test]
fn test_os_string() {
    set_up();
    let os_string = OsString::from("This is an OsString error");
    let build_run_err = ThagError::OsString(os_string.clone());
    match build_run_err {
        ThagError::OsString(os_str) => {
            assert_eq!(os_str, os_string);
        }
        _ => panic!("Expected ThagError::OsString variant"),
    }
}

#[test]
fn test_display() {
    set_up();
    let build_run_err = ThagError::Command("Command error occurred");
    assert_eq!(format!("{}", build_run_err), "Command error occurred\n");
}

#[test]
fn test_source() {
    set_up();
    let io_err = io::Error::new(io::ErrorKind::Other, "I/O error occurred");
    let build_run_err: ThagError = io_err.into();
    assert!(build_run_err.source().is_some());
}

#[test]
fn test_cancelled() {
    set_up();
    let build_run_err = ThagError::Cancelled;
    match build_run_err {
        ThagError::Cancelled => (),
        _ => panic!("Expected ThagError::Cancelled variant"),
    }
}
