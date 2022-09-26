use std::{ffi::CString};

use rust_shell::{main, parse, Command};

#[test]
fn test_simple_parse() {
    assert_eq!(parse(">foo bar < zog | wc -l"), vec!(
        Command {
            command: vec!(CString::new("bar").unwrap()),
            stdout: Some(CString::new("foo").unwrap()),
            stdin: Some(CString::new("zog").unwrap()),
        },
        Command {
            command: vec!(CString::new("wc").unwrap(), CString::new("-l").unwrap()),
            stdin: None,
            stdout: None,
        },
    ));
}
