fn main() {
    let family: String = std::env::var("CARGO_CFG_TARGET_FAMILY").unwrap_or(String::new());
    if family != "unix" {
        panic!("This program only supports Unix-like systems!");
    }
}

