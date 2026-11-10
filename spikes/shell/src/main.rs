use nsh_platform_macos::{run_app, AppOptions, AppSource, Root};

fn main() {
    spike_shell::mark_process_start();
    let mut options = AppOptions {
        title: "SuperReader spike".into(),
        size: (1100, 1400),
        fps: false,
        ..AppOptions::default()
    };
    options.size = spike_shell::opening_size();
    spike_shell::register(&options.components);
    run_app(
        AppSource::Rust(Root::of::<spike_shell::SpikeApp>()),
        options,
    );
}
