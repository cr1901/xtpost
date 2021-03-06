use ::scraper::Html;
use eyre::{Report, Result};
use futures::{future::try_join4, StreamExt, TryFutureExt, TryStreamExt};
use reqwest::{multipart, Body, Client};
use tokio::{fs::File, runtime, task::LocalSet};
use tokio_util::codec::{BytesCodec, FramedRead, FramedWrite};

use std::path::PathBuf;
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

    match args.cmd {
        // Don't bother doing work if all we want is version string.
        // Also unbreaks CI :D!
        args::SubCommands::Version(_) => {}
        _ => {
            cfg::write_cfg_if_doesnt_exist()?;
            cfg::make_data_dir_if_doesnt_exist()?;
        }
    }

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

            rt.block_on(talk_to_xt(r, cfg))?;
        }
    }

    Ok(())
}

async fn talk_to_xt(r: args::RunArgs, cfg: cfg::Config) -> Result<()> {
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
        .file_name(r.binary.clone());
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
    let resp = client.post(&cfg.server).multipart(form).send().await?;

    if r.debug {
        for h in resp.headers() {
            println!("{}: {}", h.0, h.1.to_str()?);
        }
        println!("{}", &resp.text().await?);
    } else {
        // TODO: I'd like to do an async streaming scraper where each task shares the
        // input received text (via Rc<String>?). Main benefit would be that serial
        // text is printed out as it's received, rather than all at once.
        let text = &resp.text().await?;
        let scraper = scraper::Scraper::new(Html::parse_document(text), cfg.server)?;
        let scraper_rc = Rc::new(scraper);

        let local = LocalSet::new();
        local
            .run_until(async move {
                let serial_rc = scraper_rc.clone();
                let img_rc = scraper_rc.clone();
                let file_rc = scraper_rc.clone();
                let audio_rc = scraper_rc.clone();

                // Serial capture
                let serial_task = tokio::task::spawn_local(async move {
                    println!("Server and serial text:");
                    println!("{}", &serial_rc.serial_text());
                });

                // Image capture
                let image_task =
                    tokio::task::spawn_local(get_file(client.clone(), r.image, move || {
                        img_rc.image_url()
                    }));

                // File download
                let file_task =
                    tokio::task::spawn_local(get_file(client.clone(), r.file, move || {
                        file_rc.file_url()
                    }));

                // Audio download
                let audio_task =
                    tokio::task::spawn_local(get_file(client.clone(), r.audio, move || {
                        audio_rc.audio_url()
                    }));

                let (_, img_ret, file_ret, audio_ret) =
                    try_join4(serial_task, image_task, file_task, audio_task).await?;

                // TODO: Perhaps all errors can be returned, rather than going in order?
                match img_ret? {
                    Some(filename) => println!("Image file at: {}", filename.to_str().unwrap()),
                    None => println!("No image file found."),
                }

                match file_ret? {
                    Some(filename) => println!("Captured file at: {}", filename.to_str().unwrap()),
                    None => println!("No captured file found."),
                }

                match audio_ret? {
                    Some(filename) => println!("Captured audio at: {}", filename.to_str().unwrap()),
                    None => println!("No captured audio found."),
                }

                Ok::<(), Report>(())
            })
            .await?;
    }

    Ok::<(), Report>(())
}

async fn get_file<F>(
    client: Client,
    filename_override: Option<String>,
    get_url_fn: F,
) -> Result<Option<PathBuf>>
where
    F: FnOnce() -> Result<Option<String>>,
{
    let url = match get_url_fn()? {
        Some(u) => u,
        None => return Ok(None),
    };

    let resp = client.get(&url).send().await?;
    let bytes_stream = resp
        .bytes_stream()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::ConnectionAborted, e));

    let filename =
        filename_override.map_or_else(|| cfg::url_to_data_dir(&url), |s| Ok(PathBuf::from(s)))?;
    let file_sink = File::create(&filename)
        .map_ok(|file| FramedWrite::new(file, BytesCodec::new()))
        .flatten_sink();

    bytes_stream.forward(file_sink).await?;

    Ok(Some(filename))
}
