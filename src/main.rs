use std::fmt::Display;
use std::path::PathBuf;
use clap::Parser;
use clap::Subcommand;
use fern::colors::ColoredLevelConfig;
use log::{LevelFilter, trace};
use strum::EnumString;

#[derive(Parser)]
struct ToolChainArgs {
    #[clap(long)]
    log_level: ToolChainLogLevel,
    #[clap(long)]
    color_policy: ColorPolicy,
    #[clap(long)]
    log_file: Option<PathBuf>,
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

#[derive(EnumString, Eq, PartialEq, Copy, Clone)]
#[strum(serialize_all = "camelCase")]
enum ColorPolicy {
    Always,
    Auto,
    Never,
}

impl ColorPolicy {
    fn determine(self, stream: atty::Stream) -> bool {
        match self {
            ColorPolicy::Always => true,
            ColorPolicy::Auto => atty::is(stream),
            ColorPolicy::Never => false,
        }
    }
}

#[derive(EnumString, Eq, PartialEq, Copy, Clone)]
#[strum(serialize_all = "camelCase")]
enum ToolChainLogLevel {
    Off,
    Error,
    Warning,
    Info,
    Debug,
    Trace,
}

fn setup_logger(log_level: ToolChainLogLevel, do_color: bool, output_file: Option<PathBuf>) -> Result<(), fern::InitError> {
    let color = ColoredLevelConfig::new();

    let mut x = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                if do_color {
                    Box::new(color.color(record.level())) as Box<dyn Display>
                } else {
                    Box::new(record.level()) as Box<dyn Display>
                },
                message
            ))
        })
        .level(match log_level {
            ToolChainLogLevel::Off => LevelFilter::Off,
            ToolChainLogLevel::Error => LevelFilter::Error,
            ToolChainLogLevel::Warning => LevelFilter::Warn,
            ToolChainLogLevel::Info => LevelFilter::Info,
            ToolChainLogLevel::Debug => LevelFilter::Debug,
            ToolChainLogLevel::Trace => LevelFilter::Trace,
        })
        .chain(std::io::stderr());

    if let Some(log_file) = output_file {
        x = x.chain(fern::log_file("output.log")?);
    }

    x.apply()?;
    Ok(())
}

fn main() {
    let args: ToolChainArgs = ToolChainArgs::parse();
    setup_logger(args.log_level, args.color_policy.determine(atty::Stream::Stdout), args.log_file)
        .unwrap_or_default();
    trace!("Hello!");

    trace!("Bye!");
}
