# rust shell

## Description

This is a console written in Rust. It can execute simple commands, redirect input and output streams, execute a pipeline

---

## Getting started

```console
$ cargo build -r
$ cd target/release
$ ./rust_shell
```

Examples:

```console
$ ls -la
$ cat file.txt > 1.txt
$ echo hello | tail -n 10 | wc -c
```

---

## Tests

```console
$ cargo test
```
