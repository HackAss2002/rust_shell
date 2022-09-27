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
    let mut words = Vec::<String>::new();
    let mut word = String::new();
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
    if !command.command.is_empty() {
        commands.push(command);
    }
    return commands;
}

pub fn executeCommands(commands: &Vec<Command>) {
    if commands.is_empty() {
        return;
    }
    let mut pipelines = Vec::<(Option<i32>, Option<i32>)>::with_capacity(commands.len());
    for i in 0..commands.len() {
        pipelines.push((None, None));
    }
    for i in 0..commands.len() - 1 {
        let (inpipe, outpipe) = pipe().unwrap();
        pipelines[i].1 = Option::Some(outpipe);
        pipelines[i + 1].0 = Option::Some(inpipe);
    }

    if commands.first().unwrap().stdin != None {
        unsafe { pipelines.first_mut().unwrap_unchecked() }.0 = Option::Some(
            open(
                commands
                    .first()
                    .unwrap()
                    .stdin
                    .as_ref()
                    .unwrap()
                    .to_str()
                    .unwrap(),
                OFlag::O_RDONLY,
                Mode::from_bits_truncate(0o777),
            )
            .unwrap(),
        );
    }
    if commands.last().unwrap().stdout != None {
        unsafe { pipelines.last_mut().unwrap_unchecked() }.1 = Option::Some(
            open(
                commands
                    .last()
                    .unwrap()
                    .stdout
                    .as_ref()
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
        forks.push(unsafe { fork().unwrap() });
        match forks[i] {
            ForkResult::Child => {
                let mut args = Vec::<CString>::with_capacity(commands[i].command.len());
                for command in &commands[i].command {
                    args.push(CString::new(command.clone()).unwrap());
                }

                if pipelines[i].0 != None {
                    dup2(unsafe { pipelines[i].0.unwrap_unchecked() }, 0).unwrap();
                }
                if pipelines[i].1 != None {
                    dup2(unsafe { pipelines[i].1.unwrap_unchecked() }, 1).unwrap();
                }

                for pipeline in &pipelines {
                    if pipeline.0 != None {
                        close(unsafe { pipeline.0.unwrap_unchecked() }).unwrap();
                    }
                    if pipeline.1 != None {
                        close(unsafe { pipeline.1.unwrap_unchecked() }).unwrap();
                    }
                }

                execvp(&args[0], args.as_slice()).unwrap();
            }
            ForkResult::Parent { child } => {}
        }
    }

    for pipeline in &pipelines {
        if pipeline.0 != None {
            close(unsafe { pipeline.0.unwrap_unchecked() }).unwrap();
        }
        if pipeline.1 != None {
            close(unsafe { pipeline.1.unwrap_unchecked() }).unwrap();
        }
    }

    for fork in &forks {
        match fork {
            ForkResult::Child => {}
            ForkResult::Parent { child } => {
                waitpid(*child, None).unwrap();
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
