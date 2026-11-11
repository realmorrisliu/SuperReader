use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("fixtures/apple"));
    let which = env::args().nth(2).unwrap_or_else(|| "apple".into());
    match which.as_str() {
        "apple" => spike_library::fixture::write_apple(&target),
        "calibre" => spike_library::fixture::write_calibre(&target),
        "form" => {
            let _ = spike_library::fixture::write_form(&target);
        }
        _ => {
            eprintln!("usage: library-fixture [dir] [apple|calibre|form]");
            std::process::exit(1);
        }
    }
}
