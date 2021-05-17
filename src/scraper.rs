use eyre::{eyre, Report};
use scraper::{Html, Selector};
use reqwest::{Error, IntoUrl};
use url::Url;

/// This `Scraper` is specific to reenigne's server, but there's no reason
/// why it can't be spun off into a trait later.
pub struct Scraper {
    document: Html,
    server: Url,
}

impl Scraper {
    pub fn new<T>(document: Html, server: T) -> Result<Self, Error> where T: IntoUrl {
        Ok(Scraper {
            document,
            server: server.into_url()?
        })
    }

    pub fn image_url(&self) -> Result<Option<String>, Report> {
        let img = Selector::parse("img").unwrap();

        if let Some(i) = self.document.select(&img).next() {
            let rel_path = i.value()
                            .attr("src")
                            .ok_or(eyre!("img element did not have a src attribute"))?;

            Ok(Some(self.server.join(rel_path)?.into()))
        } else {
            Ok(None)
        }
    }

    pub fn serial_text(&self) -> String {
        let p_or_pre = Selector::parse("p, pre").unwrap();

        let mut out = String::new();

        for txt in self.document.select(&p_or_pre) {
            // Turn an interator of all text in children elements into
            // a single String; this is basically a concat.
            let curr_txt: String = txt.text().collect();
            out.push_str(&curr_txt);
            out.push('\n'); // Newline is NOT implicit in concat text.
            out.push('\n'); // Add another one as a separator.
        }

        out
    }
}

#[cfg(test)]
mod test {
    use super::{Html, Scraper};

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
        <img src=\"../d5JESctuMKW88L-e.png\"/>\n\
        \n\
        Program ended normally.</pre>\n\
        <p>This concludes your XT server session.</p>\n\
        </body>\n\
        </html>";

    fn mk_doc() -> Scraper {
        Scraper::new(Html::parse_document(SAMPLE_OUTPUT), "http://reenigne.mooo.com:8088/cgi-bin/xtcancel.exe").unwrap()
    }

    #[test]
    fn test_scrape_serial_text() {
        let doc = mk_doc();
        let out = doc.serial_text();

        assert_eq!(
            "The XT Server has received your file.\n\
                    \n\
                    Your program is starting\n\
                    Resetting\n\
                    Transferring attempt 0\n\
                    Upload complete.\n\
                    Hello, World!\n\
                    \n\
                    \n\
                    \n\
                    Program ended normally.\n\
                    \n\
                    This concludes your XT server session.\n\n",
            out
        );
    }

    #[test]
    fn test_scrape_image_url() {
        let doc = mk_doc();
        let out = doc.image_url();

        assert_eq!("http://reenigne.mooo.com:8088/d5JESctuMKW88L-e.png", out.unwrap().unwrap());
    }
}
