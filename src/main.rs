use eyre::Result;
use reqwest::blocking::{multipart, Client};

mod args;
mod cfg;

fn main() -> Result<()> {
    let args: args::XtPostArgs = argh::from_env();

    cfg::write_cfg_if_doesnt_exist()?;

    match args.cmd {
        args::SubCommands::Cfg(c) => {
            if c.edit {
                cfg::open_editor()?;
                return Ok(());
            }

            if c.print_dirs {
                println!("Config Dir: {}", cfg::config_dir_name()?.to_string_lossy());
                println!("Data Dir: {}", cfg::data_dir_name()?.to_string_lossy());
                println!();
            }

            if c.print_cfg {
                let cfg = cfg::read_cfg_string()?;
                println!("{}", cfg);
            }
        }
        args::SubCommands::Version(_) => {
            println!(
                "{} {} ({})",
                env!("CARGO_CRATE_NAME"),
                env!("VERGEN_BUILD_SEMVER"),
                env!("VERGEN_GIT_SHA_SHORT")
            );
        }
        args::SubCommands::Run(r) => {
            let cfg = cfg::read_cfg()?;
            let form: multipart::Form;

            if let Some(e) = cfg.email {
                form = multipart::Form::new()
                    .text("email", e)
                    .file("binary", r.binary)?;
            } else {
                form = multipart::Form::new().file("binary", r.binary)?;
            }

            let client = Client::new();
            let resp = client.post(cfg.server).multipart(form).send()?;

            println!("{}", resp.text()?);
        }
    }

    Ok(())
}
