use std::fs::{read_dir, File};
use std::io::{Result, Write};

static TARGET_PATH: &str = "../rCore/user/target/riscv64gc-unknown-none-elf/release/";

fn main() {
    let mut f = File::create("file_test.S").unwrap();
    let mut apps: Vec<_> = read_dir("../rCore/user/src/bin")
        .unwrap()
        .into_iter()
        .map(|dir_entry| {
            let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
            name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());
            name_with_ext
        }).collect();
    apps.sort();
    writeln!(
        f,
        r#"
    .align 3
    .section .data
    .global _num_app
_num_app:
    .quad {}"#,
        apps.len()
    );

    for i in 0..apps.len() {
        writeln!(f, r#"     .quad app_{}_start"#, i);
    }
    writeln!(f, r#"     .quad app_{}_end"#, apps.len() - 1);

    writeln!(
        f,
        r#"
    .global _app_names
_app_names:"#
    );

    for app in apps.iter() {
        writeln!(f, r#"     .string "{}""#, app);
    }
    
    for (idx, app) in apps.iter().enumerate() {
        println!("app_{}: {}", idx, app);
        writeln!(
            f,
            r#"
        .section .data
        .global app_{0}_start
        .global app_{0}_end
        .align 3
app_{0}_start:
        .incbin "{2}{1}"
app_{0}_end"#,
            idx, app, TARGET_PATH
        );
    }
}
