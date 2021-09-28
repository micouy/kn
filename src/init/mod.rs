use crate::args::Shell;

pub fn init(shell: Shell, exclude_old_pwd: bool) -> String {
    match shell {
        Shell::Fish => {
            let query_command = if exclude_old_pwd {
                "_kn query --exclude \"$dirprev[-1]\" --abbr \"$argv\""
            } else {
                "_kn query --abbr \"$argv\""
            };

            format!(
                include_str!("../../init/kn.fish"),
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
                include_str!("../../init/kn.zsh"),
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
                include_str!("../../init/kn.bash"),
                query_command = query_command
            )
        }
    }
}
