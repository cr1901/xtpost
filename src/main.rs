use eyre::Result;

mod args;
mod cfg;

fn main() -> Result<()> {
    let args: args::XtPostArgs = argh::from_env();

    cfg::write_cfg_if_doesnt_exist()?;

    match args.cmd {
        args::SubCommands::Cfg(c) => {
            if c.edit {
                cfg::open_editor()?;
                return Ok(())
            }

            if c.print_dirs {
                unimplemented!();
            }

            if c.print_cfg {
                unimplemented!();
            }
        },
        args::SubCommands::Version(_) => {
            println!("{} {} ({})", env!("CARGO_CRATE_NAME"), env!("VERGEN_BUILD_SEMVER"), env!("VERGEN_GIT_SHA_SHORT"));
        },
        _ => unimplemented!()
    }

    Ok(())
}
