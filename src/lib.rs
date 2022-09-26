#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::{error::Error, ffi::CString, io, io::prelude::*, str};

#[derive(Debug, PartialEq, Eq)]
pub struct Command {
    pub command: Vec<CString>,
    pub stdin: Option<CString>,
    pub stdout: Option<CString>,
}

pub fn scanWords(line: &str) -> Vec<String> {
    let mut words: Vec<String> = Vec::<String>::new();
    let mut word: String = String::new();
    for symb in line.chars() {
        if symb == '>' || symb == '<' || symb == '|' {
            if !word.is_empty() {
                words.push(word);
                word = String::new();
            }
            words.push(symb.to_string());
        } else if symb == ' ' || symb == '\t' {
            if !word.is_empty() {
                words.push(word);
                word = String::new();
            }
        } else {
            word.push(symb);
        }
    }
    if !word.is_empty() {
        words.push(word);
    }
    return words;
}

pub fn parse(line: &str) -> Vec<Command> {
    let mut commands: Vec<Command> = Vec::<Command>::new();
    let mut command: Command = Command {
        command: Vec::<CString>::new(),
        stdin: None,
        stdout: None,
    };
    let words = scanWords(line);
    let mut i = 0;
    while i < words.len() {
        if words[i] == ">" {
            i += 1;
            command.stdout = Option::Some(CString::new(words[i].clone()).unwrap());
        } else if words[i] == "<" {
            i += 1;
            command.stdin = Option::Some(CString::new(words[i].clone()).unwrap());
        } else if words[i] == "|" {
            commands.push(command);
            command = Command {
                command: Vec::<CString>::new(),
                stdin: None,
                stdout: None,
            };
        } else {
            command.command.push(CString::new(words[i].clone()).unwrap());
        }
        i += 1;
    }
    commands.push(command);
    return commands;
}

pub fn main() -> Result<(), Box<dyn Error>> {
    for line in io::stdin().lock().lines() {
        let line = line?;
        let commands = parse(&line);
    }

    Ok(())
}
