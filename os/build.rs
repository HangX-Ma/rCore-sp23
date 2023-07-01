
use std::fs::{read_dir, File};
use std::io::{Result, Write};

static TARGET: &str = "../user/target/riscv64gc-unknown-none-elf/release/";

fn main() {
    println!("cargo:rerun-if-changed=../user/src/");
    println!("cargo:rerun-if-changed={}", TARGET);
    link_user_bins().unwrap();
}

fn link_user_bins() -> Result<()> {
    let mut f = File::create("src/link_app.S")?;
    let mut apps: Vec<String> = read_dir("../user/src/bin")?
        .into_iter()
        .filter_map(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                let file_name = path.file_name()
                    .unwrap()
                    .to_str()
                    .unwrap();
                let name_stripped = file_name.strip_suffix(".rs").unwrap().to_string();
                // filter the rust file with prefix 'test'
                if let Some(ch) = option_env!("TEST") {
                    if ch.len() == 1 && ch.chars().into_iter().all(|c| c.is_numeric()) &&
                        (name_stripped.starts_with(("test".to_string() + ch).as_str()) || 
                        name_stripped.starts_with(("ch".to_string() + ch).as_str())) {
                        return Some(name_stripped);
                    }
                }
                if name_stripped.starts_with("test") || name_stripped.starts_with("ch") {
                    return None;
                }
                // If we don't set TEST env option, we will not contain any bin files except a symbol file.
                return Some(name_stripped);
            }
            None
        })
        .collect();
    // sort bin files alphabetically 
    apps.sort();

    // avoid size error
    let apps_size: usize = apps.len();
    if apps_size < 1 {
        return Ok(());
    }

    writeln!(f, 
        r#"
    .align 3
    .section .data
    .global _num_app
_num_app:
    .quad {}"#,
    apps_size)?;

    for idx in 0..=apps_size {
        if idx != apps_size {
            writeln!(f,r#"    .quad app_{}_start"#, idx).unwrap();
        } else {
            writeln!(f,r#"    .quad app_{}_end"#, idx - 1).unwrap();
        }
    }

    for idx in 0..apps_size {
        writeln!(f, r#"
    .section .data
    .global app_{0}_start
    .global app_{0}_end
app_{0}_start:
    .incbin "{1}{2}.bin"
app_{0}_end:"#, idx, TARGET, apps.get(idx).unwrap())?;
    }

    Ok(())
}