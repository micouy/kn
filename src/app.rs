//! Command line interface logic.

use clap::{App, AppSettings, Arg, SubCommand};

// TODO: Use consts instead of str literals.

/// Creates [`clap::App`](clap::App).
pub fn app() -> App<'static, 'static> {
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
                        .possible_values(&["fish", "bash", "zsh"])
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("query")
                .help("Query for path matching the abbreviation.")
                .arg(
                    Arg::with_name("ABBR")
                        .help("\"ABBR\" itself is an abbreviation.")
                        .index(1)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("interactive")
                .help("Query for path matching the abbreviation.")
                .arg(
                    Arg::with_name("TMP_FILE")
                        .help("Temporary file to write the found path to.")
                        .index(1)
                        .required(true),
                ),
        )
}
