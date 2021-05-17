use eyre::{Report, Result};
use futures::TryFutureExt;
use reqwest::{multipart, Body, Client};
use ::scraper::Html;
use tokio::{fs::File, runtime, task::LocalSet};
use tokio_util::codec::{BytesCodec, FramedRead};

use std::rc::Rc;

mod args;
mod cfg;
mod scraper;

static APP_USER_AGENT: &str = concat!(
    env!("CARGO_CRATE_NAME"),
    "/",
    env!("VERGEN_BUILD_SEMVER"),
    "/",
    env!("VERGEN_GIT_SHA_SHORT")
);

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
            let rt = runtime::Builder::new_current_thread()
                .enable_time()
                .enable_io()
                .build()?;

            rt.block_on(async {
                // File length is required to avoid chunked transfer encoding, which XT
                // server doesn't appear to support.
                let file_len = std::fs::metadata(&r.binary)?.len();
                let file_stream = File::open(r.binary.clone())
                    .map_ok(|file| {
                        FramedRead::new(file, BytesCodec::new())
                        // Taken from: https://gist.github.com/Ciantic/aa97c7a72f8356d7980756c819563566
                        // what does this do?
                        // .map_ok(BytesMut::freeze)
                    })
                    .try_flatten_stream();

                let file_body = Body::wrap_stream(file_stream);
                let file_part = multipart::Part::stream_with_length(file_body, file_len)
                    .mime_str("application/octet-stream")?
                    .file_name(r.binary);
                // .file_name(&r.binary);
                // Borrow value does not live long enough?

                let form: multipart::Form;
                if let Some(e) = cfg.email {
                    form = multipart::Form::new()
                        .text("email", e)
                        .part("binary", file_part);
                } else {
                    form = multipart::Form::new().part("binary", file_part);
                }

                let client = Client::builder().user_agent(APP_USER_AGENT).build()?;
                let resp = client.post(cfg.server).multipart(form).send().await?;

                if r.debug {
                    for h in resp.headers() {
                        println!("{}: {}", h.0, h.1.to_str()?);
                    }
                    println!("{}", &resp.text().await?);
                } else {
                    let text = &resp.text().await?;
                    let scraper = scraper::Scraper::new(Html::parse_document(text));
                    let scraper_rc = Rc::new(scraper);

                    let local = LocalSet::new();
                    local.run_until(async move {
                        let serial_rc = scraper_rc.clone();

                        tokio::task::spawn_local(async move {
                            println!("{}", &serial_rc.serial_text());
                        }).await?;

                        Ok::<(), Report>(())
                    }).await?;
                }

                Ok::<(), Report>(())
            })?;
        }
    }

    Ok(())
}
