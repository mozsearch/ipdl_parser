extern crate ipdl_parser;

use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;

const BASE_PATH: [&'static str; 2] = [".", "tests"];
const OK_PATH: &'static str = "ok";
const ERROR_PATH: &'static str = "error";

// These tests are in error/ but will pass because the required checking
// has not yet been implemented.
const ERROR_PASS_TESTS: &'static [&'static str] = &[
    // Sync message checking hasn't been implemented.
    "PasyncMessageListed.ipdl",
    "unknownSyncMessage.ipdl",
];

// These tests are in error/ and will fail, but with the wrong error
// message.
const WRONG_ERROR_TESTS: &'static [&'static str] = &[
    // We're missing the multiple declaration check from bug 1657504, but we
    // still end up failing with weird C++ redeclaration errors.
    "PDouble.ipdl",
];

fn file_expected_error(file_name: &PathBuf) -> Vec<String> {
    let mut errors = Vec::new();
    let f = File::open(file_name).unwrap();

    for line in BufReader::new(f).lines() {
        if line.as_ref().unwrap().starts_with("//error:") {
            errors.push(line.unwrap().split_off(2));
        }
    }
    assert!(
        errors.len() > 0,
        "Test file should contain expected errors."
    );
    errors
}

// XXX This does not run efficiently. If A includes B, then we end up
// testing A and B two times each. At least for the non-error case we
// should be able to do them all together.

fn test_files(test_file_path: &str, should_pass: bool) {
    let mut path: PathBuf = BASE_PATH.iter().collect();
    path.push(test_file_path);

    let mut include_dirs = vec![path.clone()];

    // Failing protocols can reference non-failing protocols in the
    // "extra" subdirectory.
    if !should_pass {
        let mut extra_path = path.clone();
        extra_path.push("extra");
        include_dirs.push(extra_path);
    }

    let mut error_pass_tests = HashSet::new();
    for f in ERROR_PASS_TESTS {
        error_pass_tests.insert(OsStr::new(f));
    }
    let mut wrong_error_tests = HashSet::new();
    for f in WRONG_ERROR_TESTS {
        wrong_error_tests.insert(OsStr::new(f));
    }

    let entries = fs::read_dir(&path).expect("Should have the test file directory");
    for entry in entries {
        if let Ok(entry) = entry {
            // Ignore subdirectories.
            if entry.path().is_dir() {
                continue;
            }

            let mut expected_result = should_pass;
            if !should_pass && error_pass_tests.contains(entry.path().file_name().unwrap()) {
                println!(
                    "Expecting test to pass when it should fail {:?}",
                    entry.file_name()
                );
                expected_result = true;
            } else {
                println!("Testing {:?}", entry.file_name());
            }

            let file_name = vec![entry.path()];
            match ipdl_parser::compiler::compile(&include_dirs, file_name) {
                Ok(()) => assert!(expected_result, "Expected test to fail, but it passed"),
                Err(actual_error) => {
                    assert!(
                        !expected_result,
                        "Expected test to pass, but it failed with \"{}\"",
                        actual_error
                    );
                    if wrong_error_tests.contains(entry.path().file_name().unwrap()) {
                        println!(
                            "Not checking the correctness of the error message for {:?}",
                            entry.file_name()
                        );
                        continue;
                    }
                    for expected_error in file_expected_error(&entry.path()) {
                        // Lexer errors are different in lalrpop than in Ply,
                        // so do some translation so that the dtorReserved.ipdl
                        // error message passes.
                        if expected_error
                            .find("lexically invalid characters")
                            .is_some()
                        {
                            assert!(
                                actual_error.find("Unexpected token").is_some(),
                                "Expected \"Unexpected token\" in \"{}\"",
                                actual_error
                            );
                        } else {
                            assert!(
                                actual_error.find(&expected_error).is_some(),
                                "Expected \"{}\" in \"{}\"",
                                expected_error,
                                actual_error
                            );
                        }
                    }
                }
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
