#[allow(dead_code)]
use crate::{Error, Result};


use clap::ArgMatches;


pub fn init(matches: &ArgMatches<'_>) -> Result<String> {
    // Fail silently?
    let shell = matches
        .value_of("shell")
        .ok_or(dev_err!("required `clap` arg absent"))?;

    let script = match shell {
        "fish" => include_str!("../../init/kn.fish.template"),
        "bash" => include_str!("../../init/kn.bash.template"),
        "zsh" => include_str!("../../init/kn.zsh.template"),
        "powershell" => include_str!("../../init/kn.powershell.template"),
        _ => return Err(Error::InvalidArgValue("shell".to_string())),
    };


    Ok(script.to_string())
}
