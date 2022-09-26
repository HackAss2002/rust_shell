#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::error::Error;

use rust_shell;

fn main() -> Result<(), Box<dyn Error>> {
    rust_shell::main()
}
