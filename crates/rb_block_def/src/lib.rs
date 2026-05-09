mod parse;
mod code_gen;
use code_gen::generate;
use parse::parse_file;


pub fn generate_blocks(block_def: &str) -> Result<String, std::io::Error> {
    let (_, ir) = parse_file(block_def).map_err(|e| std::io::Error::other(e.to_owned()))?;
    let code = generate(&ir);
    Ok(code)
}
