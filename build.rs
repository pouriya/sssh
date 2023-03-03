use std::fs;
use std::io::BufRead;

fn main() {
    fs::read("Cargo.toml")
        .unwrap()
        .lines()
        .for_each(|maybe_line| {
            if let Ok(line) = maybe_line {
                if line.starts_with("APPLICATION_VERSION") {
                    let word_list: Vec<_> = line.split(' ').collect();
                    let version = word_list[word_list.len() - 1].replace(['"', '\''], "");
                    println!("cargo:rustc-env=APPLICATION_VERSION={}", version);
                }
            }
        })
}
