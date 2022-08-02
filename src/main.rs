use std::path::PathBuf;
use clap::Parser;
use clap::Subcommand;

#[derive(Parser)]
struct ToolChainArgs {
    #[clap(long)]
    log_level: ToolChainLogLevel,
    #[clap(subcommand)]
    sub_command: ToolChainSubCommand,
}

#[derive(Subcommand)]
enum ToolChainSubCommand {
    Compress {
        path: PathBuf,
    },
    Decompress {
        path: PathBuf,
    },
    Compile {
        source_file: PathBuf,
    },
    GenerateStub {
        json_file: PathBuf,
    },
    DumpJson {
        json_file: PathBuf,
    },
    DumpAst {
        source_file: PathBuf,
    },
}

fn main() {
    println!("Hello, world!");
}
