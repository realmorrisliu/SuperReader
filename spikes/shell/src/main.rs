use nsh_platform_macos::{run_app, AppOptions, AppSource, Appearance, Root};

fn main() {
    spike_shell::publish_launch_file();
    spike_shell::mark_process_start();
    let mut options = AppOptions {
        title: "Slowpage".into(),
        size: (1100, 1400),
        fps: false,
        // The paper is always white. Aqua keeps template icons and the
        // system chrome dark when the Mac is in Dark Mode.
        appearance: Appearance::Light,
        ..AppOptions::default()
    };
    options.size = spike_shell::opening_size();
    spike_shell::install_document_open();
    spike_shell::register(&options.components);
    run_app(
        AppSource::Rust(Root::of::<spike_shell::SpikeApp>()),
        options,
    );
}
