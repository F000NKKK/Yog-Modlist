//! Code-behind of `stat_bar.yui`: what its bindings call.

yog_api::ui_include!("stat_bar");

use yog_api::Units;

impl StatBar {
    fn fraction(&self) -> f32 {
        self.value.clamp(0.0, 1.0)
    }

    /// `Loaded 75%`, or just the label.
    fn caption(&self) -> String {
        if self.percent {
            format!("{} {}%", self.label, (self.fraction() * 100.0).round())
        } else {
            self.label.clone()
        }
    }

    /// The filled part's width.
    fn filled(&self) -> Units {
        Units::Percent(self.fraction() * 100.0)
    }
}
