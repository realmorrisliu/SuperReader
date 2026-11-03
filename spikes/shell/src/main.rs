use nsh_platform_macos::{AppOptions, AppSource, Root, run_app};

fn main() {
    let mut options = AppOptions {
        title: "SuperReader spike".into(),
        size: (1100, 1400),
        fps: false,
        ..AppOptions::default()
    };
    if let Some(size) = spike_shell::opening_size() {
        options.size = size;
    }
    spike_shell::register(&options.components);
    run_app(AppSource::Rust(Root::of::<spike_shell::SpikeApp>()), options);
}
