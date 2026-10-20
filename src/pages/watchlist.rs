//! The watchlist: the day's breadth with its donut, every symbol's performance on one indexed
//! chart over the shared range, then one rich card per tracked symbol (name, sparkline, price,
//! and a colored change chip), reconciled by `each` (rows keep their state as the list changes)
//! and tappable through to the symbol's detail page.

use crate::charts;
use crate::quotes;
use crate::res;
use day::prelude::*;

/// The chip's text in the current display mode, sign folded into the number:
/// `+1.24 (0.62%)` / `−3.10%` / `+1.24`.
pub fn change_text(q: &quotes::Quote, mode: quotes::ChipMode) -> String {
    let sign = if q.change() >= 0.0 { "+" } else { "−" };
    let (abs, pct) = (q.change().abs(), q.change_pct().abs());
    match mode {
        quotes::ChipMode::Both => format!("{sign}{abs:.2} ({pct:.2}%)"),
        quotes::ChipMode::Percent => format!("{sign}{pct:.2}%"),
        quotes::ChipMode::Absolute => format!("{sign}{abs:.2}"),
    }
}

/// The change chip: white text on the trend color, rounded, the Stocks signature. What it
/// reads (absolute, percent, or both) is app-wide state driven by [`chip_mode_button`], so
/// every chip on every page changes together.
pub fn change_chip(
    quote: Signal<day::reactive::Load<quotes::Quote>>,
    id: day::ElementId,
) -> impl Piece {
    let mode = quotes::chip_mode();
    label(move || {
        quote.with(|l| {
            l.ready()
                .map(|q| change_text(q, mode.get()))
                .unwrap_or_default()
        })
    })
    .font(Font::Callout)
    .color(Color::WHITE)
    .id(id)
    .padding(Insets::symmetric(8.0, 3.0))
    .background(move || {
        quote.with(|l| {
            l.ready()
                .map(|q| charts::trend_color(q.change() >= 0.0))
                .unwrap_or(Color::rgba(0.5, 0.5, 0.5, 0.4))
        })
    })
    .corner_radius(7.0)
}

/// The chip-mode control. Apple Stocks cycles this by tapping a chip, which does not work
/// here: a watchlist chip sits inside a row whose own tap navigates to the detail page, so a
/// tap would have to both cycle and navigate. An explicit button says what the next reading
/// will be and belongs to no row.
fn chip_mode_button() -> impl Piece {
    let mode = quotes::chip_mode();
    button(move || match mode.get() {
        quotes::ChipMode::Both => res::str::chip_both().format(),
        quotes::ChipMode::Percent => res::str::chip_percent().format(),
        quotes::ChipMode::Absolute => res::str::chip_absolute().format(),
    })
    .action(quotes::cycle_chip_mode)
    .id(day::id!(chip_mode))
}

/// Today's breadth over the whole watchlist: how many symbols are up, how many down, and the
/// day's biggest mover each way, which is the summary a list of rows cannot give at a glance.
fn breadth_strip(list: Signal<Vec<String>>) -> impl Piece {
    // Both lines take closures: the mover captions carry a live percentage, so a
    // once-formatted `LocalizedText` would freeze at the placeholder the first build saw.
    let cell = |value: Box<dyn Fn() -> String>,
                caption: Box<dyn Fn() -> String>,
                tint: Option<Color>,
                id: day::ElementId| {
        let v = label(value)
            .font(Font::Subheadline)
            .bold()
            .apply_some(tint, |v, c| v.color(c));
        column((v.id(id), label(caption).font(Font::Caption2)))
            .spacing(1.0)
            .align(HAlign::Leading)
            .grow_w()
    };
    // The mover cells carry the symbol as their value and the percentage in the caption. Both
    // on the value line wraps to two lines on a phone, which drops that cell's caption below
    // the other three and leaves the strip on a ragged baseline.
    let mover_symbol = |best: bool| {
        let f = move || {
            quotes::breadth(&list.get())
                .and_then(|b| if best { b.best } else { b.worst })
                .map(|(s, _)| s)
                .unwrap_or_else(|| "—".to_string())
        };
        Box::new(f) as Box<dyn Fn() -> String>
    };
    let mover_pct = move |best: bool| {
        quotes::breadth(&list.get())
            .and_then(|b| if best { b.best } else { b.worst })
            .map(|(_, pct)| format!("{pct:+.2}%"))
            .unwrap_or_else(|| "—".to_string())
    };
    row((
        // The same two counts as a ring, so the day's balance reads before the numbers do.
        charts::breadth_donut(list).id(day::id!(breadth_donut)),
        cell(
            Box::new(move || {
                quotes::breadth(&list.get())
                    .map(|b| b.up.to_string())
                    .unwrap_or_else(|| "—".to_string())
            }),
            Box::new(|| res::str::breadth_up().format()),
            Some(charts::trend_color(true)),
            day::id!(breadth_up),
        ),
        cell(
            Box::new(move || {
                quotes::breadth(&list.get())
                    .map(|b| b.down.to_string())
                    .unwrap_or_else(|| "—".to_string())
            }),
            Box::new(|| res::str::breadth_down().format()),
            Some(charts::trend_color(false)),
            day::id!(breadth_down),
        ),
        cell(
            mover_symbol(true),
            Box::new(move || res::str::breadth_best(mover_pct(true)).format()),
            None,
            day::id!(breadth_best),
        ),
        cell(
            mover_symbol(false),
            Box::new(move || res::str::breadth_worst(mover_pct(false)).format()),
            None,
            day::id!(breadth_worst),
        ),
    ))
    .spacing(10.0)
    .align(VAlign::Center)
    .card()
    .id(day::id!(breadth))
}

/// Every symbol's move over the shared range on one axis, each indexed to 100 at the window's
/// start, so the lines read as relative performance whatever the instruments' prices. The range
/// picker is the detail page's own signal: the range chosen here is the one a symbol opens on.
fn performance_card(list: Signal<Vec<String>>) -> impl Piece {
    let title = label(res::str::performance_title()).font(Font::Headline);
    let range_picker = picker(quotes::range(), charts::range_options())
        .segmented()
        .id(day::id!(wl_range_picker));
    // The title and a five-segment picker do not fit side by side on a phone; there they stack.
    let header = adaptive((title, spacer(), range_picker))
        .spacing(Space::S)
        .grow_w();
    // Two selections, one per chart: each is written and read by the chart that owns it, and the
    // readouts below say in words what the guides say in pixels.
    let risk_sel = Signal::new(None);
    let corr_sel = Signal::new(None);
    let readout = |sel: Signal<Option<day_piece_charts::Selection>>,
                   none: fn() -> day::LocalizedText| {
        label(move || match sel.get() {
            Some(s) => {
                let v = s.values.first();
                res::str::wl_risk_readout(res::str::WlRiskReadout {
                    ret: v.map(|v| v.label.clone()).unwrap_or_default(),
                    symbol: v.map(|v| v.series.clone()).unwrap_or_default(),
                    vol: s.x_label.clone(),
                })
                .format()
            }
            None => none().format(),
        })
        .font(Font::Caption)
    };
    column((
        header,
        charts::performance_chart(list).id(day::id!(performance_chart)),
        // Risk against return: the comparison a price line cannot make.
        label(res::str::wl_risk_title()).font(Font::Headline),
        charts::risk_return_scatter(list, risk_sel).id(day::id!(risk_return)),
        readout(risk_sel, res::str::wl_risk_none).id(day::id!(risk_readout)),
        // How the list moves together.
        label(res::str::wl_corr_title()).font(Font::Headline),
        charts::correlation_matrix(list, corr_sel).id(day::id!(correlation)),
        readout(corr_sel, res::str::wl_corr_none).id(day::id!(correlation_readout)),
    ))
    .spacing(Space::S)
    .align(HAlign::Leading)
    .card()
    .id(day::id!(performance))
}

fn row_card(symbol: String) -> impl Piece {
    let quote = quotes::resource_for(&symbol).signal();
    let nav_to = symbol.clone();
    let title = symbol.clone();
    row((
        column((
            label(title).font(Font::Headline).bold(),
            label(move || quote.with(|l| l.ready().map(|q| q.name.clone()).unwrap_or_default()))
                .font(Font::Footnote),
        ))
        .spacing(2.0)
        .align(HAlign::Leading)
        .grow_w(),
        charts::sparkline(quote),
        column((
            label(move || {
                quote.with(|l| {
                    l.ready()
                        .map(|q| format!("{:.2}", q.last))
                        .unwrap_or_else(|| "…".to_string())
                })
            })
            .font(Font::Title3)
            .bold()
            .id(day::id!(wl_price, &symbol)),
            change_chip(quote, day::id!(wl_chg, &symbol)),
        ))
        .spacing(3.0)
        .align(HAlign::Trailing),
    ))
    .spacing(Space::M)
    .card()
    .on_tap(move || {
        crate::open_symbol(&nav_to);
    })
    .id(day::id!(wl_row, symbol))
}

pub fn watchlist_page() -> impl Piece {
    let list = quotes::symbols();
    let sort = quotes::sort();

    scroll(
        column((
            label(res::str::watchlist_title())
                .font(Font::Title)
                .bold()
                .id(day::id!(watchlist_title)),
            label(move || match quotes::source() {
                quotes::DataSource::Mock => res::str::data_mock().format(),
                quotes::DataSource::Demo => res::str::data_demo().format(),
                quotes::DataSource::Live => res::str::data_live().format(),
            })
            .font(Font::Caption)
            .id(day::id!(data_source)),
            super::demo_notice(),
            // Empty watchlist: say so and point at the page that fixes it, rather than
            // rendering a bare heading over nothing.
            when(
                move || list.get().is_empty(),
                || {
                    column((
                        label(res::str::watchlist_empty()).font(Font::Callout),
                        label(res::str::watchlist_empty_hint()).font(Font::Footnote),
                    ))
                    .spacing(Space::XS)
                    .align(HAlign::Leading)
                    .card()
                    .id(day::id!(watchlist_empty))
                },
            ),
            when(move || !list.get().is_empty(), move || breadth_strip(list)),
            when(
                move || !list.get().is_empty(),
                move || performance_card(list),
            ),
            // List-wide controls, laid out for the width available. Side by side needs room
            // for a three-segment picker and the chip button; at compact width that overflows
            // (on a 392dp phone the localized segments alone eat the row and the button lands
            // off-screen), so there they stack instead.
            when(
                move || !list.get().is_empty(),
                move || {
                    let sorter = picker(
                        quotes::SortBinding,
                        [
                            (quotes::Sort::Manual, res::str::sort_manual()),
                            (quotes::Sort::Name, res::str::sort_name()),
                            (quotes::Sort::Change, res::str::sort_change()),
                        ],
                    )
                    .segmented()
                    .id(day::id!(sort_picker));
                    adaptive((sorter, spacer(), chip_mode_button()))
                        .spacing(Space::S)
                        .grow_w()
                },
            ),
            each(
                items(
                    move || quotes::sorted_symbols(list.get(), sort.get()),
                    |s| s.clone(),
                ),
                |slot| row_card(slot.key()),
            ),
            label(res::str::data_attribution()).font(Font::Caption2),
        ))
        .spacing(10.0)
        .align(HAlign::Leading)
        .padding(Space::L),
    )
    .grow()
}
