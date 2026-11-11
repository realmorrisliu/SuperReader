use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let svg_path = args
        .next()
        .ok_or("usage: spike-icon <svg> <iconset> <preview>")?;
    let iconset = args
        .next()
        .ok_or("usage: spike-icon <svg> <iconset> <preview>")?;
    let preview = args
        .next()
        .ok_or("usage: spike-icon <svg> <iconset> <preview>")?;
    let svg = std::fs::read_to_string(&svg_path).map_err(|err| format!("{svg_path}: {err}"))?;
    spike_icon::write_set(&svg, &PathBuf::from(iconset), &PathBuf::from(preview))
}
