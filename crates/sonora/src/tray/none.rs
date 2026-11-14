//! The tray icon on platforms that have none.
//!
//! Android has no status notifier area, so the tray is always absent: `Icon::new` answers
//! `None`, `install` therefore reports that the app does not linger, and the rows about the
//! tray icon stay inert. Playback state on Android belongs in the media session instead.

use anyhow::Result;
use tokio::sync::mpsc::UnboundedSender;

use super::{Event, Shown};

pub struct Icon;

impl Icon {
    pub fn new(_sender: UnboundedSender<Event>) -> Option<Self> {
        None
    }

    pub fn hosted() -> bool {
        false
    }

    pub fn place(&mut self, _placed: bool) -> Result<()> {
        Ok(())
    }

    pub fn show(&mut self, _shown: &Shown) {}
}
