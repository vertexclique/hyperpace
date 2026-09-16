//! Macro library commands: the saved macro store, optionally mirrored to a device macro slot.

use hyperpace_device::DeviceHandle;
use hyperpace_protocol::{MacroEvent, MacroEventKind, MacroSlot, offset};
use tauri::{AppHandle, Manager};

use crate::blocking::blocking;
use crate::dto::{MacroRecordDto, SaveMacroRequest};
use crate::error::{AppError, to_command_result};
use crate::state::AppState;

/// Save a macro: create a new document when `request.id` is `None`, otherwise update the
/// existing one. When `request.slot` is set and a device is connected, also writes the macro to
/// that device macro slot; a device write failure is reported (a slot was explicitly asked for),
/// but no device connected at all is not an error, since the store record is this library's own
/// durable copy regardless of whether anything is plugged in right now.
///
/// # Errors
///
/// Returns an error message when the store rejects the write, `request.id` names no existing
/// macro, or a device write was attempted and the connection is read-only or did not
/// acknowledge it in time.
#[tauri::command]
pub async fn save_macro(
    app: AppHandle,
    request: SaveMacroRequest,
) -> Result<MacroRecordDto, String> {
    to_command_result(
        blocking(move || {
            let state = app.state::<AppState>();
            let record = request.to_record();
            let id = match &request.id {
                Some(id) => {
                    state.store.macros().update(id, &record)?;
                    id.clone()
                }
                None => state.store.macros().create(&record)?,
            };

            if let Some(slot) = request.slot {
                match state.handle() {
                    Ok(handle) => write_macro_slot(&handle, slot, &request)?,
                    Err(AppError::NotConnected) => {}
                    Err(other) => return Err(other),
                }
            }

            Ok(MacroRecordDto::from_record(id, record))
        })
        .await,
    )
}

/// Every saved macro.
///
/// # Errors
///
/// Returns an error message when the store cannot be reached.
#[tauri::command]
pub async fn list_macros(app: AppHandle) -> Result<Vec<MacroRecordDto>, String> {
    to_command_result(
        blocking(move || {
            let state = app.state::<AppState>();
            let records = state.store.macros().list()?;
            Ok(records
                .into_iter()
                .map(|(id, record)| MacroRecordDto::from_record(id, record))
                .collect())
        })
        .await,
    )
}

/// Delete a saved macro. Returns `true` when a macro was deleted, `false` when `id` already
/// named none.
///
/// # Errors
///
/// Returns an error message when the store cannot be reached.
#[tauri::command]
pub async fn delete_macro(app: AppHandle, id: String) -> Result<bool, String> {
    to_command_result(
        blocking(move || {
            let state = app.state::<AppState>();
            Ok(state.store.macros().delete(&id)?)
        })
        .await,
    )
}

/// Encode `request`'s events as a [`MacroSlot`] and write it to device macro slot `slot`.
fn write_macro_slot(
    handle: &DeviceHandle,
    slot: u8,
    request: &SaveMacroRequest,
) -> Result<(), AppError> {
    let macro_slot = MacroSlot {
        name: request.name.clone(),
        events: request.events.iter().map(to_macro_event).collect(),
    };
    let bytes = macro_slot.encode()?;
    handle.write_block(offset::MACRO + u16::from(slot) * 384, &bytes)?;
    Ok(())
}

fn to_macro_event(event: &crate::dto::MacroEventDto) -> MacroEvent {
    MacroEvent {
        press: event.press,
        kind: kind_from_string(&event.kind),
        value: event.value,
        delay_ms: event.delay_ms,
    }
}

/// The string this crate stores [`MacroEventKind`] under
/// ([`hyperpace_store::MacroEventRecord::kind`]), decoded back to the protocol type a device
/// write needs. Not `Display`/`FromStr` on the protocol type itself: that crate depends on
/// nothing outside `core`/`std` (`docs/architecture/api-contract.md`), so the string form lives
/// here, the one place that needs it. Never fails: an unrecognized or malformed string decodes as
/// [`MacroEventKind::Other(0)`] rather than reject a record this crate itself did not write.
fn kind_from_string(kind: &str) -> MacroEventKind {
    match kind {
        "Modifier" => MacroEventKind::Modifier,
        "Key" => MacroEventKind::Key,
        "Consumer" => MacroEventKind::Consumer,
        "Mouse" => MacroEventKind::Mouse,
        other => other
            .strip_prefix("Other(")
            .and_then(|rest| rest.strip_suffix(')'))
            .and_then(|n| n.parse::<u8>().ok())
            .map_or(MacroEventKind::Other(0), MacroEventKind::Other),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use hyperpace_device::{Access, SimTransport};
    use hyperpace_protocol::offset;

    use crate::dto::MacroEventDto;

    #[test]
    fn every_named_kind_string_decodes_to_its_variant() {
        assert_eq!(kind_from_string("Modifier"), MacroEventKind::Modifier);
        assert_eq!(kind_from_string("Key"), MacroEventKind::Key);
        assert_eq!(kind_from_string("Consumer"), MacroEventKind::Consumer);
        assert_eq!(kind_from_string("Mouse"), MacroEventKind::Mouse);
        assert_eq!(kind_from_string("Other(9)"), MacroEventKind::Other(9));
    }

    #[test]
    fn a_malformed_kind_string_falls_back_instead_of_panicking() {
        assert_eq!(kind_from_string("nonsense"), MacroEventKind::Other(0));
        assert_eq!(
            kind_from_string("Other(not-a-number)"),
            MacroEventKind::Other(0)
        );
    }

    #[test]
    fn writing_a_macro_slot_lands_at_the_documented_offset() {
        let (transport, _controller) = SimTransport::new(62, 1, 4);
        let handle = hyperpace_device::spawn(Box::new(transport), Access::ReadWrite).unwrap();

        let request = SaveMacroRequest {
            id: None,
            name: "burst".to_owned(),
            slot: Some(2),
            events: vec![MacroEventDto {
                press: true,
                kind: "Key".to_owned(),
                value: 4,
                delay_ms: 10,
            }],
        };
        write_macro_slot(&handle, 2, &request).unwrap();

        let shadow = handle.read_settings().unwrap();
        let base = offset::MACRO + 2 * 384;
        let slot_bytes: Vec<u8> = (0..384).map(|i| shadow.scalar(base + i)).collect();
        let decoded = MacroSlot::decode(&slot_bytes).unwrap().unwrap();
        assert_eq!(decoded.name, "burst");
        assert_eq!(decoded.events.len(), 1);
        assert_eq!(decoded.events[0].kind, MacroEventKind::Key);
    }
}
