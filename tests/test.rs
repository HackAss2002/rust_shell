use std::{ffi::CString};

use core::time;

use std::{process::exit, thread::sleep};

use std::sync::Mutex;

use nix::{
    errno::Errno,
    fcntl::{fcntl, FcntlArg, OFlag},
    libc::{STDIN_FILENO, STDOUT_FILENO},
    sys::{
        wait::{waitpid, WaitStatus},
    },
    unistd::{close, dup2, fork, pipe, read, write, ForkResult},
};

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

static SHELL_LOCK: Mutex<()> = Mutex::new(());

fn run_shell<F>(mut cb: F)
where
    F: FnMut(&mut dyn FnMut(String) -> (), &mut dyn FnMut() -> String) -> (),
{
    let _guard = SHELL_LOCK.lock().unwrap();

    let (inpipe, outchild) = pipe().unwrap();
    let (inchild, outpipe) = pipe().unwrap();

    fcntl(inpipe, FcntlArg::F_SETFL(OFlag::O_NONBLOCK)).unwrap();

    let pid = match unsafe { fork().unwrap() } {
        ForkResult::Parent { child } => child,
        ForkResult::Child => {
            close(inpipe).unwrap();
            close(outpipe).unwrap();

            dup2(outchild, STDOUT_FILENO).unwrap();
            close(outchild).unwrap();

            dup2(inchild, STDIN_FILENO).unwrap();
            close(inchild).unwrap();

            main().unwrap();
            exit(0);
        },
    };

    let mut send = |s: String| {
        assert_eq!(write(outpipe, s.as_bytes()).unwrap(), s.len());
    };

    let mut recv = || -> String {
        let mut buf = Vec::new();
        buf.resize(1024, 0);

        sleep(time::Duration::from_millis(1000));
        let n = match read(inpipe, buf.as_mut_slice()) {
            Ok(n) => n,
            Err(Errno::EWOULDBLOCK) => 0,
            e => e.unwrap(),
        };

        buf.resize(n, 0);

        String::from_utf8(buf).unwrap()
    };

    cb(&mut send, &mut recv);

    close(outpipe).unwrap();

    assert_eq!(waitpid(pid, None).unwrap(), WaitStatus::Exited(pid, 0));
}

#[test]
fn test_simple_shell() {
    run_shell(|send, recv| {
        send("echo hello\n".to_string());
        assert_eq!(recv(), "hello\n".to_string());

        send("touch /tmp/foo\n".to_string());
        assert_eq!(recv(), "".to_string());

        send("ls /tmp/foo\n".to_string());
        assert_eq!(recv(), "/tmp/foo\n".to_string());
    })
}
