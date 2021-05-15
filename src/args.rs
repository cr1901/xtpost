use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// XT Server HTTP Client
pub struct XtPostArgs {
    #[argh(subcommand)]
    pub cmd: SubCommands,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum SubCommands {
    Version(Version),
    Cfg(CfgArgs),
    Run(RunArgs),
}

#[derive(FromArgs, PartialEq, Debug)]
/// print version info
#[argh(subcommand, name = "version")]
pub struct Version {}

#[derive(FromArgs, PartialEq, Debug)]
/// edit or view configuration directory (edit takes priority)
#[argh(subcommand, name = "cfg")]
pub struct CfgArgs {
    #[argh(switch, short = 'e')]
    /// edit settings.json
    pub edit: bool,
    /// print settings.json to stdout
    #[argh(switch, short = 'c')]
    pub print_cfg: bool,
    #[argh(switch, short = 'd')]
    /// print config directories
    pub print_dirs: bool,
}

#[derive(FromArgs, PartialEq, Debug)]
/// submit data to XT server and wait for results
#[argh(subcommand, name = "run")]
pub struct RunArgs {
    #[argh(positional)]
    pub binary: String,
}
