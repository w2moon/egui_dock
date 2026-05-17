use crate::TabDestination;

/// Drop action queued on pointer release and applied after all surfaces register dock rects.
#[derive(Debug)]
pub enum PendingDrop {
    /// In-process tab drag with overlay destination from the releasing frame.
    Tab {
        destination: Option<TabDestination>,
    },
    /// Cross-viewport payload drag; destination is re-resolved at apply time when possible.
    CrossViewport {
        fallback_destination: Option<TabDestination>,
    },
}
