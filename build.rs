use imagesize;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
    // only run this if assets folder changed
    println!("cargo:rerun-if-changed=assets");

    textures();
    levels();
}

fn textures() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("texture_codegen.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    let textures = std::fs::read_dir("assets/textures").unwrap();
    let filenames = textures
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            match path.extension().and_then(|s| s.to_str()) {
                Some("png") => {
                    let size = imagesize::size(&path).unwrap();
                    Some((entry, size))
                }
                _ => None,
            }
        })
        .map(|(entry, size)| (entry.file_name().into_string().unwrap(), size))
        .collect::<Vec<_>>();

    writeln!(
        file,
        "pub const TEXTURE_FILENAMES: &[(&str, (usize, usize))] = &["
    )
    .unwrap();
    for (filename, size) in filenames.iter() {
        writeln!(
            file,
            r#"    ("{}", ({}, {})),"#,
            filename, size.width, size.height
        )
        .unwrap();
    }
    writeln!(file, "];").unwrap();
}

fn levels() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("level_codegen.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    let levels = std::fs::read_dir("assets/levels").unwrap();
    let mut paths = levels
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            match path.extension().and_then(|s| s.to_str()) {
                Some("svg") => Some(path),
                _ => None,
            }
        })
        .collect::<Vec<_>>();
    paths.sort();
    writeln!(file, "pub const LEVEL_SVGS: &[&str] = &[").unwrap();
    for path in paths {
        let data = std::fs::read_to_string(&path).unwrap();
        writeln!(file, r####"    r###"{}"###,"####, data).unwrap();
    }
    writeln!(file, "];").unwrap();
}