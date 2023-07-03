pub fn tempfs() -> (lib::Fs, tempfile::TempDir) {
    let td = tempfile::TempDir::new().unwrap();
    let fs = lib::Fs::new(td.path());
    (fs, td)
}

pub fn config() -> lib::Config {
    lib::Config {
        first_index_in_date: 0,
        lim_account_type: None,
        unsigned_is_positive: true,
        use_colored_output: false,
        use_unicode_symbols: false,
    }
}

pub struct FsState {
    config: Option<lib::Config>,
    rl: Option<lib::Recordlist>,
    limits: Option<lib::Limits>,
}
