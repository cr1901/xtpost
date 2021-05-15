use eyre::Result;

mod cfg;

fn main() -> Result<()> {
    cfg::write_cfg_if_doesnt_exist()?;
    cfg::open_editor()?;

    Ok(())
}
