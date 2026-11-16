//! Product wordmark hook.
//!
//! Upstream FilmCraft draws the ArtCraft trademark here. That mark is not part of
//! this fork (see `FORK-NOTICE.md`), so the hook renders nothing until we ship a
//! wordmark we own. Callers already treat the result as optional.

/// No wordmark is bundled in this fork.
pub fn wordmark(_ctx: &egui::Context, _dark: bool) -> Option<egui::TextureHandle> {
    None
}

/// Draw nothing. Returns `None`, so callers fall back to text-only layouts.
pub fn paint_wordmark(
    _ui: &egui::Ui,
    _left_center: egui::Pos2,
    _height: f32,
    _dark: bool,
) -> Option<egui::Rect> {
    None
}
