//! The hotplug watcher.
//!
//! hidapi has no native hotplug notification (protocol reference: "Hotplug comes from a separate
//! watcher started before enumeration, since hidapi has none"), so this polls periodically and
//! reports when the number of attached vendor collections changes. The diffing itself
//! (`diff`) is a pure, directly tested function; only the hidapi polling shell around it is
//! left unexercised by this crate's own tests, for the same reason [`crate::HidTransport`] is:
//! opening or enumerating a real device is out of scope here (workspace rule).

use std::cmp::Ordering;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use hidapi::HidApi;

use crate::error::DeviceError;
use crate::hid::is_vendor_collection;

/// How often the watcher re-enumerates.
const POLL_INTERVAL: Duration = Duration::from_millis(500);

/// A change in how many of this product's vendor collections are attached, since the watcher's
/// last look.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotplugEvent {
    /// At least one more collection is attached than before.
    Added,
    /// At least one fewer collection is attached than before.
    Removed,
}

/// The event a change from `previous` to `present` attached collections produces, or `None` when
/// the count did not change.
fn diff(previous: usize, present: usize) -> Option<HotplugEvent> {
    match present.cmp(&previous) {
        Ordering::Greater => Some(HotplugEvent::Added),
        Ordering::Less => Some(HotplugEvent::Removed),
        Ordering::Equal => None,
    }
}

/// Count how many attached collections match this product's vendor configuration channel.
fn count_present() -> Result<usize, DeviceError> {
    let api = HidApi::new().map_err(|error| DeviceError::Io(error.to_string()))?;
    Ok(api
        .device_list()
        .filter(|info| is_vendor_collection(info))
        .count())
}

/// Start the hotplug watcher and return the channel it reports on.
///
/// Call this before any enumeration of your own (before [`crate::HidTransport::open_first`], for
/// instance): a device that appears in the gap between your own initial enumeration and starting
/// a watcher would otherwise never be reported. A device already attached when `watch` is called
/// is not itself reported as [`HotplugEvent::Added`]; it is the baseline every later count is
/// compared against, so your own initial enumeration is still how you learn about it.
///
/// # Errors
///
/// Returns [`DeviceError::Io`] when the operating system could not start the watcher thread.
pub fn watch() -> Result<mpsc::Receiver<HotplugEvent>, DeviceError> {
    let (tx, rx) = mpsc::channel();
    thread::Builder::new()
        .name("hyperpace-device-hotplug".to_owned())
        .spawn(move || {
            let events = tx;
            run(&events);
        })
        .map_err(|error| DeviceError::Io(error.to_string()))?;
    Ok(rx)
}

fn run(events: &mpsc::Sender<HotplugEvent>) {
    let mut previous = count_present().unwrap_or(0);
    loop {
        thread::sleep(POLL_INTERVAL);
        let Ok(present) = count_present() else {
            continue; // hidapi could not be reached this tick; retried next tick
        };
        if let Some(event) = diff(previous, present)
            && events.send(event).is_err()
        {
            return; // every receiver dropped; nothing left to notify
        }
        previous = present;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_increase_is_added() {
        assert_eq!(diff(1, 2), Some(HotplugEvent::Added));
    }

    #[test]
    fn a_decrease_is_removed() {
        assert_eq!(diff(2, 1), Some(HotplugEvent::Removed));
    }

    #[test]
    fn no_change_is_no_event() {
        assert_eq!(diff(1, 1), None);
    }

    #[test]
    fn zero_to_present_is_added() {
        assert_eq!(diff(0, 1), Some(HotplugEvent::Added));
    }

    #[test]
    fn present_to_zero_is_removed() {
        assert_eq!(diff(1, 0), Some(HotplugEvent::Removed));
    }
}
