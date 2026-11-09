//! Manage the watchlist: add a symbol by ticker (Yahoo Finance format, e.g. `SPY`, `GC=F`),
//! add from a picker of suggestions, and remove tracked rows. Mirrors the Apple Stocks
//! edit sheet as a plain form page.

use crate::quotes;
use crate::res;
use day::prelude::*;

#[derive(Clone, PartialEq)]
enum Status {
    Idle,
    Added,
    Removed,
    Exists,
    Empty,
}

impl Status {
    fn text(&self) -> String {
        match self {
            Status::Idle => String::new(),
            Status::Added => res::str::manage_status_added().format(),
            Status::Removed => res::str::manage_status_removed().format(),
            Status::Exists => res::str::manage_status_exists().format(),
            Status::Empty => res::str::manage_status_empty().format(),
        }
    }
}

pub fn manage_page() -> impl Piece {
    let sym_list = quotes::symbols();
    let entry = Signal::new(String::new());
    let status = Signal::new(Status::Idle);
    let preset = Signal::new(quotes::PRESETS[0]);

    let add_typed = move || {
        let symbol = quotes::normalize(&entry.get_untracked());
        if symbol.is_empty() {
            status.set(Status::Empty);
            return;
        }
        status.set(if quotes::add(&symbol) {
            entry.set(String::new());
            Status::Added
        } else {
            Status::Exists
        });
    };

    // One recycling-list row per tracked symbol (drag to reorder; the order is the sidebar
    // order, persisted with the list).
    let rows = list(
        items(move || sym_list.get(), |s: &String| s.clone()),
        move |slot| {
            // Recycling rows (docs/list.md): cells rebind as the list changes or reorders, so
            // the action reads the slot's current key at click time and the id re-registers
            // reactively (`id_of`), since a build-time key would go stale.
            row((
                column((
                    label(move || slot.get()).font(Font::Headline),
                    label(move || quotes::preset_name(&slot.get()).to_string())
                        .font(Font::Footnote),
                ))
                .spacing(1.0)
                .align(HAlign::Leading)
                .grow(Dimension::Width),
                button(res::str::manage_remove())
                    .action(move || {
                        quotes::remove(&slot.key());
                        status.set(Status::Removed);
                    })
                    .id_of(move || day::id!(sym_remove, slot.get())),
            ))
            .spacing(Space::S)
            .padding(Insets::horizontal(4.0))
            // Open the symbol. Inside the Symbols tab this pushes onto that tab's own stack;
            // in the desktop sidebar it selects the row (the enclosing nav owns the route).
            .on_tap(move || {
                crate::open_symbol(&slot.key());
            })
            // Reactively, like the Remove button above: a recycled cell rebinds to another
            // symbol, and a build-time id would address the wrong row after a reorder.
            .id_of(move || day::id!(sym_row, slot.get()))
        },
    )
    .row_height(RowHeight::Uniform(52.0))
    .reorderable(Reorder::new(quotes::move_symbol))
    // Swipe a row away where the platform has the gesture (docs/list.md); the desktops answer
    // `Cap::ListDelete = Unsupported` and keep using the row's own Remove button.
    .deletable(
        Delete::new(|index| {
            let list = quotes::symbols().get_untracked();
            if let Some(sym) = list.get(index) {
                quotes::remove(sym);
            }
        })
        .label(res::str::manage_remove()),
    )
    .id(day::id!(sym_rows))
    .height(312.0);

    let presets: Vec<(&str, String)> = quotes::PRESETS
        .iter()
        .map(|s| (*s, format!("{s} — {}", quotes::preset_name(s))))
        .collect();

    scroll(
        column((
            super::demo_notice(),
            form((
                section((rows,)).title(res::str::manage_list_section()),
                section((
                    labeled(
                        res::str::manage_symbol_label(),
                        text_field(entry)
                            .placeholder("SPY".to_string())
                            .id(day::id!(sym_field)),
                    ),
                    label(res::str::manage_symbol_hint()).font(Font::Footnote),
                    button(res::str::manage_add())
                        .action(add_typed)
                        .prominent()
                        .id(day::id!(sym_add)),
                    labeled(
                        res::str::manage_preset_label(),
                        picker(preset, presets).id(day::id!(preset_picker)),
                    ),
                    button(res::str::manage_add_preset())
                        .action(move || {
                            status.set(if quotes::add(preset.get_untracked()) {
                                Status::Added
                            } else {
                                Status::Exists
                            });
                        })
                        .id(day::id!(preset_add)),
                ))
                .title(res::str::manage_add_section()),
            )),
            label(move || status.get().text())
                .font(Font::Footnote)
                .id(day::id!(sym_status)),
        ))
        .spacing(Space::M)
        .align(HAlign::Leading)
        .padding(Space::L),
    )
    .grow(Dimension::Both)
}

/// Ask for a ticker and add it. The `+` in the Symbols tab's navigation bar and the
/// desktop's Add Symbol… menu item share this one flow (docs/dialogs.md).
pub fn prompt_for_symbol() {
    day::task(async move {
        let entered = prompt(res::str::add_symbol_title())
            .message(res::str::add_symbol_body())
            // OK/Cancel come from Day's core catalog, already localized for every locale the
            // app ships (docs/dialogs.md), so no app strings are needed for them.
            .placeholder("SPY".to_string())
            .await;
        if let Some(text) = entered {
            let symbol = quotes::normalize(&text);
            if !symbol.is_empty() {
                quotes::add(&symbol);
            }
        }
    });
}

/// The desktop's Symbols menu (docs/menus.md): the toolkits with a real menu bar get Add and
/// Remove there, because they have no swipe gesture and a `+` bar button belongs to a phone's
/// navigation bar, not a desktop window's chrome.
///
/// The labels are `res::str` reads, so a runtime language switch re-lowers the menu in the new
/// language.
pub fn install_app_menu() {
    app_menu(|| {
        vec![sub_menu(
            res::str::menu_symbols(),
            vec![
                add_symbol_command().menu_item(),
                menu_item(res::str::menu_remove_symbol()).action(|| {
                    // The selected symbol IS the current route on the sidebar shell; a route
                    // that is not a tracked symbol (the watchlist, settings) removes nothing.
                    if let Some(route) = current_route() {
                        let list = quotes::symbols().get_untracked();
                        if list.contains(&route) {
                            quotes::remove(&route);
                        }
                    }
                }),
            ],
        )]
    });
}

/// Shared by the Symbols menu and the sidebar toolbar.
pub fn add_symbol_command() -> Command {
    Command::new(
        day::id!(add_symbol),
        res::str::menu_add_symbol(),
        prompt_for_symbol,
    )
    .image(res::vectors::add_symbol)
    .shortcut(Shortcut::primary("n"))
}
