use isotope_core::controller::View;

pub const SIDEBAR_WIDTH: f64 = 64.0;
pub const FOLDER_PANEL_WIDTH: f64 = 200.0;
pub const TOAST_HEIGHT: f64 = 32.0;

/// Logical-pixel rectangle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Area for the visible app webview: right of the sidebar (and folder panel),
/// above the toast strip when a toast is showing.
pub fn content_bounds(window_width: f64, window_height: f64, view: View) -> Bounds {
    let x = SIDEBAR_WIDTH + if view.folder_panel_open { FOLDER_PANEL_WIDTH } else { 0.0 };
    let height = window_height - if view.toast_visible { TOAST_HEIGHT } else { 0.0 };
    Bounds { x, y: 0.0, width: (window_width - x).max(0.0), height: height.max(0.0) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_layout_leaves_room_for_the_sidebar() {
        let b = content_bounds(1280.0, 800.0, View::default());
        assert_eq!(b, Bounds { x: 64.0, y: 0.0, width: 1216.0, height: 800.0 });
    }

    #[test]
    fn folder_panel_and_toast_shrink_the_content() {
        let view = View { folder_panel_open: true, overlay_open: false, toast_visible: true };
        let b = content_bounds(1280.0, 800.0, view);
        assert_eq!(b, Bounds { x: 264.0, y: 0.0, width: 1016.0, height: 768.0 });
    }

    #[test]
    fn tiny_windows_never_produce_negative_sizes() {
        let view = View { folder_panel_open: true, overlay_open: false, toast_visible: true };
        let b = content_bounds(100.0, 20.0, view);
        assert_eq!((b.width, b.height), (0.0, 0.0));
    }
}
