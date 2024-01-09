use imagesize;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
    // only run this if assets folder changed
    println!("cargo:rerun-if-changed=assets/textures");

    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("texture_codegen.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    // read all files in assets folder
    let assets = std::fs::read_dir("assets/textures").unwrap();
    let filenames = assets
        .map(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();
            let size = imagesize::size(&path).unwrap();
            (entry, size)
        })
        .map(|(entry, size)| (entry.file_name().into_string().unwrap(), size))
        .collect::<Vec<_>>();

    // write as const array
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
