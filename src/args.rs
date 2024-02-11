use clap::Parser;
use clap::ArgGroup;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(group(
ArgGroup::new("mode")
.required(true)
.args(["pack", "unpack"]),
))]
pub struct Args {
    /// Source Folder to unpack / pack from
    #[arg(short, long)]
    pub source: String,

    /// Destination Folder to unpack / pack to
    #[arg(short, long)]
    pub destination: String,

    /// Unpack Source Folder to Destination Folder
    #[arg(short, long)]
    pub unpack: bool,

    /// Pack Source Folder to Destination Folder
    #[arg(short, long)]
    pub pack: bool,

    /// Path to the Path.json File (defaults to searching in program directory)
    #[arg(long)]
    pub path_json: Option<String>
}