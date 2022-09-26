#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::{error::Error, ffi::CString, io, io::prelude::*, str};

use nix::{
    fcntl::{open, OFlag},
    libc::{O_WRONLY, STDIN_FILENO, STDOUT_FILENO},
    sys::{stat::Mode, wait::waitpid},
    unistd::{close, dup2, execvp, fork, pipe, ForkResult},
};

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
            command
                .command
                .push(CString::new(words[i].clone()).unwrap());
        }
        i += 1;
    }
    commands.push(command);
    return commands;
}

pub fn executeCommands(commands: &Vec<Command>) {
    let mut pipeline = Vec::<(Option<i32>, Option<i32>)>::with_capacity(commands.len());
    for i in 0..commands.len() {
        pipeline.push((None, None));
    }
    for i in 0..commands.len() - 1 {
        let (inpipe, outpipe) = pipe().unwrap();
        pipeline[i].1 = Option::Some(outpipe);
        pipeline[i + 1].0 = Option::Some(inpipe);
    }

    if commands.first().unwrap().stdin != None {
        pipeline.first_mut().unwrap().0 = Option::Some(
            open(
                commands
                    .first()
                    .unwrap()
                    .stdin
                    .clone()
                    .unwrap()
                    .to_str()
                    .unwrap(),
                OFlag::O_CREAT | OFlag::O_WRONLY,
                Mode::from_bits_truncate(0o777),
            )
            .unwrap(),
        );
    }
    if commands.last().unwrap().stdout != None {
        pipeline.last_mut().unwrap().1 = Option::Some(
            open(
                commands
                    .last()
                    .unwrap()
                    .stdout
                    .clone()
                    .unwrap()
                    .to_str()
                    .unwrap(),
                OFlag::O_CREAT | OFlag::O_WRONLY,
                Mode::from_bits_truncate(0o777),
            )
            .unwrap(),
        );
    }

    let mut forks = Vec::<ForkResult>::with_capacity(commands.len());
    for i in 0..commands.len() {
        if forks.len() == 0 || forks.last().unwrap().is_parent() {
            forks.push(unsafe { fork().unwrap() });
            match forks[i] {
                ForkResult::Child => {
                    let mut args: Vec<CString> = Vec::<CString>::new();
                    for j in 0..commands[i].command.len() {
                        args.push(CString::new(commands[i].command[j].clone()).unwrap());
                    }
                    if pipeline[i].0 != None {
                        dup2(pipeline[i].0.unwrap(), 0).unwrap();
                    }
                    if pipeline[i].1 != None {
                        dup2(pipeline[i].1.unwrap(), 1).unwrap();
                    }
                    for j in 0..pipeline.len() {
                        if pipeline[j].0 != None {
                            close(pipeline[j].0.unwrap()).unwrap();
                        }
                        if pipeline[j].1 != None {
                            close(pipeline[j].1.unwrap()).unwrap();
                        }
                    }
                    execvp(&args[0], args.as_slice()).unwrap();
                    break;
                }
                ForkResult::Parent { child } => {}
            }
        }
    }
    for j in 0..pipeline.len() {
        if pipeline[j].0 != None {
            close(pipeline[j].0.unwrap()).unwrap();
        }
        if pipeline[j].1 != None {
            close(pipeline[j].1.unwrap()).unwrap();
        }
    }

    for i in 0..forks.len() {
        match forks[i] {
            ForkResult::Child => {}
            ForkResult::Parent { child } => {
                waitpid(child, None).unwrap();
            }
        }
    }
}

pub fn main() -> Result<(), Box<dyn Error>> {
    for line in io::stdin().lock().lines() {
        let line = line?;
        let commands = parse(&line);
        executeCommands(&commands);
    }

    Ok(())
}
