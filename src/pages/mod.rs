//! One module per navigation destination (https://daybrite.dev/docs/navigation), plus the one
//! piece of chrome every destination carries: the demo-data notice.

use crate::quotes;
use crate::res;
use day::prelude::*;

mod detail;
mod manage;
mod settings;
mod watchlist;

pub use detail::detail_page;
pub use manage::{install_app_menu, manage_page, prompt_for_symbol};
pub use settings::{apply_startup, settings_page};
pub use watchlist::watchlist_page;

/// The notice every page shows while the app is reading its bundled snapshots instead of live
/// quotes: the web build with no proxy configured, or the demo-data setting turned on.
///
/// Tinted and bold rather than a quiet caption, because prices that are months old look exactly
/// like today's until something says otherwise. It reads `quotes::source()`, so turning the
/// setting on or filling in a proxy adds or removes it in place.
pub fn demo_notice() -> impl Piece {
    when(
        || quotes::source() == quotes::DataSource::Demo,
        || {
            label(|| {
                if quotes::demo().get() || quotes::is_demo_env() {
                    res::str::demo_notice_chosen().format()
                } else {
                    res::str::demo_notice_no_proxy().format()
                }
            })
            .font(Font::Callout)
            .bold()
            // `.id` before the wrapping decorators, so the script's assert lands on the label
            // rather than on the padding node.
            .id("demo-notice")
            .padding(Insets::symmetric(10.0, 12.0))
            .background(Color::rgba(0.95, 0.65, 0.10, 0.22))
            .corner_radius(10.0)
        },
    )
}
