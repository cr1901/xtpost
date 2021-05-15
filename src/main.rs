use eyre::Result;
use reqwest::blocking::{multipart, Client};
use scraper::{Html, Selector};

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

            println!("{}", scrape_text(&resp.text()?));
        }
    }

    Ok(())
}

fn scrape_text(inp: &str) -> String {
    let document = Html::parse_document(inp);
    let p_or_pre = Selector::parse("p, pre").unwrap();

    let mut out = String::new();

    for txt in document.select(&p_or_pre) {
        let curr_txt: String = txt.text().collect();
        out.push_str(&curr_txt);
        out.push('\n');
        out.push('\n');
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

        assert_eq!("The XT Server has received your file.\n\
                    \n\
                    Your program is starting\n\
                    Resetting\n\
                    Transferring attempt 0\n\
                    Upload complete.\n\
                    Hello, World!\n\
                    \n\
                    Program ended normally.\n\
                    \n\
                    This concludes your XT server session.\n\n", out);
    }
}
