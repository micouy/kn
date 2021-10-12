//! The `init` subcommand.

use crate::args::Shell;

/// The `init` subcommand.
///
/// Prints a shell script for initializing `kn`. The script
/// can be configured. The `init` subcommand takes an arg `--shell`,
/// specifying the used shell, and a flag `--exclude-old-pwd` which
/// enables excluding the previous location from the search (only if there
/// are other matching dirs).
pub fn init(shell: Shell, exclude_old_pwd: bool) -> String {
    match shell {
        Shell::Fish => {
            let query_command = if exclude_old_pwd {
                "_kn query --exclude \"$dirprev[-1]\" --abbr \"$argv\""
            } else {
                "_kn query --abbr \"$argv\""
            };

            format!(
                include_str!("../init/kn.fish"),
                query_command = query_command
            )
        }
        Shell::Zsh => {
            let query_command = if exclude_old_pwd {
                "_kn query --exclude \"${OLDPWD}\" --abbr \"$@\""
            } else {
                "_kn query --abbr \"$@\""
            };

            format!(
                include_str!("../init/kn.zsh"),
                query_command = query_command
            )
        }
        Shell::Bash => {
            let query_command = if exclude_old_pwd {
                "_kn query --exclude \"${OLDPWD}\" --abbr \"$@\""
            } else {
                "_kn query --abbr \"$@\""
            };

            format!(
                include_str!("../init/kn.bash"),
                query_command = query_command
            )
        }
    }
}
