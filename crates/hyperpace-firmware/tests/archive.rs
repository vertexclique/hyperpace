//! Proves the container parser and the identity gate against real sibling firmware images,
//! archived read-only under `firmware/archive/other-devices/`.
//!
//! Every image referenced here belongs to another vendor's product on the same ODM platform, not
//! to the HYPACE (`firmware/archive/SOURCES.md`: "no such image was found anywhere"); the field
//! values asserted below are copied from that ledger, independently recomputed while writing this
//! crate. Every one must classify as [`Match::ForeignDevice`], which is exactly what these tests
//! check. No file here is opened for anything but a read.

// This is a test binary, not a production path: the doctrine's no-panic rule (`CLAUDE.md`
// section 3) reserves panics for tests, and an integration test crate is not gated by a
// `#[cfg(test)] mod tests` the way this crate's own unit tests are, so the same allowances are
// declared explicitly here.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use hyperpace_protocol::{DeviceIdentity, LinkType};

use hyperpace_firmware::{
    DeviceType, Match, Package, RECEIVER_PRODUCT_ID, UsbIds, VENDOR_ID, WIRED_PRODUCT_ID,
};

fn read_archive(relative: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../firmware/archive/other-devices")
        .join(relative);
    std::fs::read(&path).unwrap_or_else(|err| {
        panic!("reading archived firmware {}: {err}", path.display());
    })
}

fn hyperpace_wired_mouse() -> (DeviceIdentity, UsbIds) {
    (
        DeviceIdentity {
            cid: 0,
            mid: 0,
            link: LinkType::Wired1k,
        },
        UsbIds {
            vendor_id: VENDOR_ID,
            product_id: WIRED_PRODUCT_ID,
        },
    )
}

fn hyperpace_receiver_usb() -> UsbIds {
    UsbIds {
        vendor_id: VENDOR_ID,
        product_id: RECEIVER_PRODUCT_ID,
    }
}

#[test]
fn a_sibling_mouse_image_sharing_the_vendor_id_is_still_classified_foreign() {
    // firmware/archive/SOURCES.md: headCRC 0x5555273a(ok); ver v2.18; DeviceType 210 (mouse);
    // cid/mid 0/0; icName NRF52833; normal=vid_3554&pid_f507 -- the HYPACE's own USB vendor id,
    // but not its product id, so the endpoint gate (T5) must still refuse it.
    let bytes = read_archive(
        "pulsar-xlite-v3/mouse-v2.18/(2) Mouse Update v2.18.exe__res.dev0.upgrade.bin",
    );
    let pkg = Package::parse(&bytes).expect("a well-formed sibling image parses");

    assert_eq!(pkg.images.len(), 1);
    let header = &pkg.images[0];
    assert_eq!(header.head_crc, 0x5555_273a);
    assert_eq!(header.fw_len, 84_576);
    assert_eq!(header.next, 0);
    assert_eq!(header.version.to_string(), "v2.18");
    assert_eq!(header.device_type, DeviceType::Mouse);
    assert_eq!(header.cid, 0);
    assert_eq!(header.mid, 0);
    assert_eq!(header.ic_name, "NRF52833");
    assert_eq!(header.normal_endpoint.vendor_id, 0x3554);
    assert_eq!(header.normal_endpoint.product_id, 0xf507);
    assert_eq!(header.reset_cmd.len(), 17);
    assert_eq!(header.prepare_cmd.len(), 49);
    assert_eq!(header.data_cmd.len(), 49);

    let (id, usb) = hyperpace_wired_mouse();
    assert!(matches!(
        pkg.matches(&id, &usb),
        Match::ForeignDevice { .. }
    ));
}

#[test]
fn a_sibling_dongle_image_is_classified_foreign() {
    // SOURCES.md: headCRC 0x555522ff(ok); ver v3.00; DeviceType 211 (dongle); icName CX52650N;
    // normal=vid_3554&pid_f508.
    let bytes = read_archive(
        "pulsar-xlite-v3/dongle-1k-v3.00/(1) 1K Dongle Update v3.00.exe__res.dev0.upgrade.bin",
    );
    let pkg = Package::parse(&bytes).expect("a well-formed sibling image parses");

    let header = &pkg.images[0];
    assert_eq!(header.head_crc, 0x5555_22ff);
    assert_eq!(header.fw_len, 34_912);
    assert_eq!(header.version.to_string(), "v3.00");
    assert_eq!(header.device_type, DeviceType::Dongle);
    assert_eq!(header.ic_name, "CX52650N");
    assert_eq!(header.normal_endpoint.vendor_id, 0x3554);
    assert_eq!(header.normal_endpoint.product_id, 0xf508);

    let (id, _) = hyperpace_wired_mouse();
    assert!(matches!(
        pkg.matches(&id, &hyperpace_receiver_usb()),
        Match::ForeignDevice { .. }
    ));
}

#[test]
fn a_chained_sibling_package_parses_both_images_and_is_classified_foreign() {
    // SOURCES.md: image[0] CH32V305 v4.00 dongle @0, fwLength 29584, next 37776; image[1]
    // NRF52820_8K v4.00 dongle @37776; both declare normal=vid_3710&pid_5406, a fully foreign
    // vendor id, proving the chain parser (V5) and the endpoint gate together.
    let bytes = read_archive(
        "pulsar-crazylight-8k/v4.07/Xlite_CrazyLight_Medium+8K_Dongle.exe__res.dev0.upgrade.bin",
    );
    let pkg =
        Package::parse(&bytes).expect("a well-formed chained sibling package parses both images");

    assert_eq!(pkg.images.len(), 2);
    assert_eq!(pkg.images[0].head_crc, 0x5555_24f2);
    assert_eq!(pkg.images[0].fw_len, 29_584);
    assert_eq!(pkg.images[0].next, 37_776);
    assert_eq!(pkg.images[0].ic_name, "CH32V305");
    assert_eq!(pkg.images[1].head_crc, 0x5555_22e2);
    assert_eq!(pkg.images[1].fw_len, 17_216);
    assert_eq!(pkg.images[1].next, 0);
    assert_eq!(pkg.images[1].ic_name, "NRF52820_8K");
    for header in &pkg.images {
        assert_eq!(header.device_type, DeviceType::Dongle);
        assert_eq!(header.normal_endpoint.vendor_id, 0x3710);
        assert_eq!(header.normal_endpoint.product_id, 0x5406);
    }

    let (id, _) = hyperpace_wired_mouse();
    assert!(matches!(
        pkg.matches(&id, &hyperpace_receiver_usb()),
        Match::ForeignDevice { .. }
    ));
}
