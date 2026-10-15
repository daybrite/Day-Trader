//! Settings: the Data group of the shared settings surface (https://daybrite.dev/docs/settings):
//! the proxy template quote fetches go through, the demo-data switch, and the refresh action.
//! Appearance, language and About come from day-piece-settings itself (`lib.rs`).

use crate::quotes;
use crate::res;
use day::prelude::*;
use day_piece_settings::prelude::*;

/// The Data group: a page of the Preferences window on desktop, a row of the settings root on
/// a phone. Every row is bound to its `Pref` in `quotes.rs`, so an edit persists as it is made
/// and `quotes::watch_source_settings` refetches on the change.
pub fn data_group() -> Group {
    Group::new("data", res::str::settings_data_section())
        .icon(Symbol::Refresh)
        // The proxy every quote fetch is routed through. Edited as a template rather than a
        // host, because the two proxy families take the target differently: a placeholder in
        // a query parameter, or a prefix the target is appended to (quotes::proxied). The app
        // ships no proxy, so this is empty until someone fills it in; while it is empty, the
        // web build reads the bundled snapshots. "Fetch directly" is the reset: the key leaves
        // the store and the field empties with it.
        .section(
            Section::new()
                .title(res::str::settings_proxy_section())
                .row(
                    rows::text(&quotes::PROXY, res::str::settings_proxy_label())
                        // An example, not a default: nothing is sent anywhere until the field
                        // holds a template the user chose.
                        .placeholder("https://relay.example/raw?url=%u")
                        .id("proxy-field"),
                )
                .row(
                    rows::action(res::str::settings_proxy_direct(), || quotes::PROXY.reset())
                        .id("proxy-direct"),
                )
                .footer(res::str::settings_proxy_hint()),
        )
        // The bundled snapshots, on demand and on any platform. The web build turns them on by
        // itself while the field above is empty; this makes them a choice everywhere.
        .section(
            Section::new()
                .row(rows::toggle(&quotes::DEMO, res::str::settings_demo_label()).id("demo-toggle"))
                .footer(res::str::settings_demo_hint()),
        )
        .section(
            Section::new()
                .row(
                    rows::action(res::str::settings_refresh(), quotes::reload_all)
                        .id("refresh-all"),
                )
                .footer(res::str::settings_refresh_hint()),
        )
}
