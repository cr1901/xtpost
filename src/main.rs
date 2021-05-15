mod cfg;

fn main() {
    cfg::write_cfg_if_doesnt_exist();
    cfg::open_editor();
}
