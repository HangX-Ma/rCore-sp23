use std::{process::Command};
fn main() {
    let output = Command::new("ls").arg("-a").output().expect("failed to execute the process");
    let file_list = String::from_utf8(output.stdout).unwrap();
    println!("{}", file_list);
}