fn main() {
    day::launch(
        day::WindowOptions {
            title: "Day Trader".into(),
            // A desktop-appropriate default size; mobile fills the screen regardless.
            size: day::prelude::Size::new(960.0, 640.0),
            // What About shows on a plain cargo build; `day build` exports the manifest's
            // identity, which day-piece-settings reads first.
            version: Some(env!("CARGO_PKG_VERSION").into()),
            ..Default::default()
        },
        dayapp::root,
    );
}
