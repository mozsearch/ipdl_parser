extern crate ipdl_parser;

use ipdl_parser::ast::TranslationUnit;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const BASE_PATH: [&'static str; 2] = [".", "tests"];
const OK_PATH: &'static str = "ok";
const ERROR_PATH: &'static str = "error";

// error/twoprotocols.ipdl is disabled because of Issue #1.

// The other tests in error/ are disabled because the given checking
// is not enabled yet. Part of the issue is that the smoke tester only
// runs the parser.

fn test_files(test_file_path: &str, should_pass: bool) {
    let mut path: PathBuf = BASE_PATH.iter().collect();
    path.push(test_file_path);

    let include_dirs = vec![path.clone()];

    let entries = fs::read_dir(&path).expect("Should have the test file directory");
    for entry in entries {
        if let Ok(entry) = entry {
            if entry.path().extension().unwrap() == "disabled" {
                assert!(!should_pass, "Expected only error tests to be disabled");
                println!("Skipping {:?}", entry.file_name());
                // XXX What should happen here is that instead of
                // continuing, we check to make sure that the test
                // passes. That way, if somebody fixes the IPDL
                // compiler, we'll get an error. However, for some
                // tests, like twoprotocols.ipdl, the error is not
                // handled gracefully. See issue #1.
                continue;
            } else {
                println!("Testing {:?}", entry.file_name());
            }

            let file_names = vec![entry.path()];
            let tus = ipdl_parser::parser::parse(&include_dirs, file_names);
            if should_pass {
                assert!(tus.is_some());
            } else {
                assert!(tus.is_none());
            }
        }
    }
}

#[test]
fn ok_tests() {
    test_files(OK_PATH, true);
}

#[test]
fn error_tests() {
    test_files(ERROR_PATH, false);
}