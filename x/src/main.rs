#![forbid(unsafe_code)]

use chrono::Local;
use env_logger::{self, fmt::Color};
use log::Level;
use std::io::Write;
use structopt::StructOpt;

mod lint;
mod playground;

type Result<T> = anyhow::Result<T>;

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(name = "lint")]
    /// Run lints
    Lint(lint::Args),
    #[structopt(name = "playground")]
    /// Run playground code
    Playground(playground::Args),
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let color = match record.level() {
                Level::Warn => Color::Yellow,
                Level::Error => Color::Red,
                _ => Color::Green,
            };

            let mut level_style = buf.style();
            level_style.set_color(color).set_bold(true);

            writeln!(
                buf,
                "{:>12} [{}] - {}",
                level_style.value(record.level()),
                Local::now().format("%T%.3f"),
                record.args()
            )
        })
        .init();

    let args = Args::from_args();

    match args.cmd {
        Command::Lint(args) => lint::run(args),
        Command::Playground(args) => playground::run(args),
    }
}
