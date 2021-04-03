//! Command line interface logic.

use clap::{App, AppSettings, Arg, SubCommand};

/// Creates [`clap::App`](clap::App).
pub fn app() -> App<'static, 'static> {
    log::trace!("create app");

    App::new(env!("CARGO_BIN_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        // Add dots at the end of messages.
        .help_message("Prints help information.")
        .version_message("Prints version information.")
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::SubcommandRequired)
        .subcommand(
            SubCommand::with_name("init")
                .help("Get init script for your shell.")
                .arg(
                    Arg::with_name("shell")
                        .possible_values(&["fish"])
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("query")
                .setting(AppSettings::TrailingVarArg)
                .help("Query directory matching given slices. If the first slice is a valid dir path, the search begins there.")
                .arg(
                    Arg::with_name("SLICES")
                        .help("Slices of path to be matched.")
                        .index(1)
                        .multiple(true)
                ),
        )
}
