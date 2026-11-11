use nsh_platform_macos::{run_app, AppOptions, AppSource, Appearance, Root};

fn root() -> Root {
    let mut root = Root::of::<spike_shell::SpikeApp>();
    // run_app does not read nsh.json. The core drops file: URLs unless the
    // root claims the file scheme, which is what nsh's generated root does
    // when fileTypes is present.
    let info: serde_json::Value =
        serde_json::from_str(include_str!("../nsh.json")).expect("nsh.json");
    if nsh_core::file_type::declares_file_types(&info) {
        root = root.url_schemes(["file"]);
    }
    root
}

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
    options.size = if spike_shell::launches_full() {
        spike_shell::shell_opening_size()
    } else {
        spike_shell::opening_size()
    };
    spike_shell::register(&options.components);
    run_app(AppSource::Rust(root()), options);
}
