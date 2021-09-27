use crate::args::Shell;

pub fn init(shell: Shell) -> &'static str {
    match shell {
        Shell::Fish => include_str!("../../init/kn.fish"),
        Shell::Zsh => include_str!("../../init/kn.zsh"),
        Shell::Bash => include_str!("../../init/kn.bash"),
    }
}
