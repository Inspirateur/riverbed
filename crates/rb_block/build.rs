use rb_block_def::generate_blocks;
use std::{env, error::Error, fs, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    let dest_path = Path::new(&env::var_os("OUT_DIR").unwrap()).join("blocks.rs");
    // CARGO_MANIFEST_DIR is the crates/rb_block directory, workspace root is two levels up
    let manifest_dir = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let workspace_dir = Path::new(&manifest_dir).join("../..").canonicalize()?;
    let block_def_path = workspace_dir.join("assets/data/blocks.def");
    let block_def = fs::read_to_string(&block_def_path)?;
    let rust_code = generate_blocks(&block_def)?;
    fs::write(&dest_path, rust_code)?;
    println!("cargo::rerun-if-changed={}", block_def_path.display());
    Ok(())
}
