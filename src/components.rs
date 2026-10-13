//! This mod's own `.yui` components (`<Pill …/>`, `<StatBar …/>`): reusable
//! widgets built from yog-ui ones, used from markup like built-in elements
//! and shown — drawn by this code — in Yog-IDLE's screen designer toolbox.

use yog_api::ui::component::{Component, Prop, Props};
use yog_api::ui::Widget;
use yog_api::{widget, Align, FlexDir, Units};

/// A small framed tag: `<Pill label="Yog" fill="#FF2D4A8A"/>`.
pub struct Pill;

impl Component for Pill {
    const NAME: &'static str = "Pill";
    const DOC: &'static str = "A small framed tag with a short label.";
    const PROPS: &'static [Prop] = &[
        Prop::text("label", "Yog").doc("The tag's text"),
        Prop::color("fill", "#FF2D4A8A").doc("Background"),
        Prop::color("outline", "#FF6F8FD8").doc("Frame color"),
        Prop::color("text_color", "#FFFFFFFF"),
    ];

    fn build(p: &Props) -> Widget {
        let inner = widget::panel(FlexDir::Row)
            .bg(p.color("fill"))
            .padding(1.0, 4.0, 1.0, 4.0)
            .align(Align::Center)
            .child(widget::label(p.text("label")).color(p.color("text_color")).shadow(false).no_wrap());
        widget::panel(FlexDir::Row).bg(p.color("outline")).padding(1.0, 1.0, 1.0, 1.0).child(inner)
    }
}

/// A labelled progress bar: `<StatBar label="Loaded" value="0.75" w="120"/>`.
pub struct StatBar;

impl Component for StatBar {
    const NAME: &'static str = "StatBar";
    const DOC: &'static str = "A labelled progress bar.";
    const PROPS: &'static [Prop] = &[
        Prop::text("label", "Progress"),
        Prop::number("value", "0.5").doc("Filled fraction, 0..1"),
        Prop::color("fill", "#FF55C055"),
        Prop::color("track", "#FF303030"),
        Prop::bool("percent", true).doc("Show the value as a percentage"),
    ];

    fn build(p: &Props) -> Widget {
        let value = p.number("value").clamp(0.0, 1.0);
        let caption = if p.bool("percent") {
            format!("{} {}%", p.text("label"), (value * 100.0).round())
        } else {
            p.text("label")
        };
        let bar = widget::panel(FlexDir::Row)
            .w(Units::Percent(100.0))
            .h(Units::Px(5.0))
            .bg(p.color("track"))
            .child(widget::spacer().w(Units::Percent(value * 100.0)).h(Units::Percent(100.0)).bg(p.color("fill")));
        widget::panel(FlexDir::Column)
            .min_w(60.0)
            .gap(2.0)
            .child(widget::label(caption).color(0xFF_DDDDDD).shadow(false).no_wrap())
            .child(bar)
    }
}

yog_api::ui_component!(Pill, StatBar);
