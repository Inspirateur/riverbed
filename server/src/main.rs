use clap::Parser;

mod asset_processing;
mod generation;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 8000)]
    port: u16,

    #[arg(short, long, default_value = "default")]
    world: String,

    #[arg(short, long)]
    game_folder_path: Option<String>,
}

fn main() {
    let args = Args::parse();
    
    unimplemented!();
}
