use eyre::{Report, Result};
use futures::TryFutureExt;
use reqwest::{multipart, Body, Client};
use scraper::{Html, Selector};
use tokio::{fs::File, runtime};
use tokio_util::codec::{BytesCodec, FramedRead};

mod args;
mod cfg;

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

                println!("{}", scrape_text(&resp.text().await?));

                Ok::<(), Report>(())
            })?;
        }
    }

    Ok(())
}

fn scrape_text(inp: &str) -> String {
    let document = Html::parse_document(inp);
    let p_or_pre = Selector::parse("p, pre").unwrap();

    let mut out = String::new();

    for txt in document.select(&p_or_pre) {
        // Turn an interator of all text in children elements into
        // a single String; this is basically a concat.
        let curr_txt: String = txt.text().collect();
        out.push_str(&curr_txt);
        out.push('\n'); // Newline is NOT implicit in concat text.
        out.push('\n'); // Add another one as a separator.
    }

    out
}

#[cfg(test)]
mod test {
    use super::scrape_text;

    static SAMPLE_OUTPUT: &str = "<!DOCTYPE html PUBLIC '-//W3C//DTD XHTML 1.0 Strict//EN' 'http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd'>\n\
        <html xmlns='http://www.w3.org/1999/xhtml' dir='ltr' lang='en-US'>\n\
        <head>\n\
        <meta http-equiv='Content-Type' content='text/html; charset=UTF-8' />\n\
        <title>XT Server - result</title>\n\
        </head>\n\
        <body><h1>XT Server</h1>\n\
        <p>The XT Server has received your file.</p>\n\
        <form action='http://reenigne.mooo.com:8088/cgi-bin/xtcancel.exe' method='post'>\n\
        <input type='hidden' name='secret' value='d5JESctuMKW88L-e'/>\n\
        <button type='submit'>Cancel</button>\n\
        </form>\n\
        <pre>Your program is starting\n\
        Resetting\n\
        Transferring attempt 0\n\
        Upload complete.\n\
        Hello, World!\n\
        \n\
        Program ended normally.</pre>\n\
        <p>This concludes your XT server session.</p>\n\
        </body>\n\
        </html>";

    #[test]
    fn test_scrape_text() {
        let out = scrape_text(SAMPLE_OUTPUT);

        assert_eq!(
            "The XT Server has received your file.\n\
                    \n\
                    Your program is starting\n\
                    Resetting\n\
                    Transferring attempt 0\n\
                    Upload complete.\n\
                    Hello, World!\n\
                    \n\
                    Program ended normally.\n\
                    \n\
                    This concludes your XT server session.\n\n",
            out
        );
    }
}
