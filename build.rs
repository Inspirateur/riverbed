use riverbed_block_def::generate_blocks;
use std::{env, error::Error, fs, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    let dest_path = Path::new(&env::var_os("OUT_DIR").unwrap()).join("blocks.rs");
    let block_def = fs::read_to_string("assets/data/blocks.def")?;
    let rust_code = generate_blocks(&block_def)?;
    fs::write(&dest_path, rust_code)?;
    println!("cargo::rerun-if-changed=assets/data/blocks.def");
    Ok(())
}