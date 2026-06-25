//! Compile-and-run guard that the `#[repr(C)]` payloads match the reverse-
//! engineered byte layouts. Sizes/offsets come from the RE dossiers; a mismatch
//! here means a drop-in break.

use std::mem::{offset_of, size_of};

use fprt_sys::ui::application as app;
use fprt_sys::ui::menu;
use fprt_sys::ui::sitehandler as site;
use fprt_sys::ui::{
    image_record::ImageRecord, layout_tuple::LayoutTuple, sld_rect::SldRect, XButton, XPiece,
    XRepresentation, XRollover,
};

#[test]
fn shared_sizes() {
    assert_eq!(size_of::<ImageRecord>(), 0x18);
    assert_eq!(size_of::<SldRect>(), 0x10);
    assert_eq!(size_of::<LayoutTuple>(), 0x14);
    assert_eq!(size_of::<XPiece>(), 0x48);
    assert_eq!(size_of::<XRollover>(), 0x20);
    assert_eq!(size_of::<XRepresentation>(), 0x48);
    assert_eq!(size_of::<XButton>(), 0x38);
}

#[test]
fn graphics_offsets() {
    assert_eq!(offset_of!(XPiece, kind), 0x10);
    assert_eq!(offset_of!(XPiece, plane), 0x18);
    assert_eq!(offset_of!(XPiece, image), 0x30);
    assert_eq!(offset_of!(XRollover, piece_count), 0x10);
    assert_eq!(offset_of!(XRollover, pieces), 0x18);
    assert_eq!(offset_of!(XRepresentation, image), 0x08);
    assert_eq!(offset_of!(XRepresentation, geom), 0x20);
    assert_eq!(offset_of!(XRepresentation, rollover_count), 0x38);
    assert_eq!(offset_of!(XRepresentation, rollovers), 0x40);
    assert_eq!(offset_of!(XButton, label), 0x08);
    assert_eq!(offset_of!(XButton, concealed), 0x18);
    assert_eq!(offset_of!(XButton, entry_state), 0x1c);
    assert_eq!(offset_of!(XButton, icon_image), 0x20);
}

#[test]
fn application_payload_sizes() {
    assert_eq!(size_of::<app::update_images::UpdateImages>(), 0x228);
    assert_eq!(size_of::<app::update_zoom::UpdateZoom>(), 0x08);
    assert_eq!(size_of::<app::update_layout::UpdateLayout>(), 0x08);
    assert_eq!(size_of::<app::update_directionality::UpdateDirectionality>(), 0x08);
    assert_eq!(size_of::<app::add_clipboard_text::AddClipboardText>(), 0x18);
    assert_eq!(size_of::<app::add_clipboard_image::AddClipboardImage>(), 0x20);
    assert_eq!(size_of::<app::open_directory::OpenDirectory>(), 0x08);
    assert_eq!(size_of::<app::launch_way_out::LaunchWayOut>(), 0x18);
    assert_eq!(size_of::<app::event_start::EventStart>(), 0x18);
    assert_eq!(size_of::<app::event_menu_access_wanted::MenuAccessWanted>(), 0x0c);
    assert_eq!(size_of::<app::event_leaptofrogans::EventLeaptofrogans>(), 0x18);
    assert_eq!(size_of::<app::event_change_layout::EventChangeLayout>(), 0x50);
    assert_eq!(size_of::<app::event_change_layout::ChangeLayoutSitehandler>(), 0x1c);
}

#[test]
fn update_images_offsets() {
    use app::update_images::UpdateImages;
    assert_eq!(offset_of!(UpdateImages, type_tag), 0x000);
    assert_eq!(offset_of!(UpdateImages, pad_main), 0x008);
    assert_eq!(offset_of!(UpdateImages, pad_anim_delay), 0x028);
    assert_eq!(offset_of!(UpdateImages, pad_anim_count), 0x030);
    assert_eq!(offset_of!(UpdateImages, pad_anim_images), 0x038);
    assert_eq!(offset_of!(UpdateImages, pad_main_discreet), 0x040);
    assert_eq!(offset_of!(UpdateImages, site_anim_delay), 0x060);
    assert_eq!(offset_of!(UpdateImages, site_anim_count), 0x068);
    assert_eq!(offset_of!(UpdateImages, site_anim_images), 0x070);
    assert_eq!(offset_of!(UpdateImages, tooltip), 0x078);
    assert_eq!(offset_of!(UpdateImages, ring_released), 0x1f8);
    assert_eq!(offset_of!(UpdateImages, ring_captured), 0x210);
}

#[test]
fn change_layout_offsets() {
    use app::event_change_layout::EventChangeLayout;
    assert_eq!(offset_of!(EventChangeLayout, sitehandlers), 0x18);
    assert_eq!(offset_of!(EventChangeLayout, pad_change_occured), 0x20);
    assert_eq!(offset_of!(EventChangeLayout, pad_layout), 0x24);
    assert_eq!(offset_of!(EventChangeLayout, menu_change_occured), 0x38);
    assert_eq!(offset_of!(EventChangeLayout, menu_layout), 0x3c);
}

#[test]
fn sitehandler_payload_sizes() {
    assert_eq!(size_of::<site::site_lifecycle::SiteLifecycle>(), 0x08);
    assert_eq!(size_of::<site::update_layout::UpdateLayout>(), 0x20);
    assert_eq!(size_of::<site::update_visual::UpdateVisual>(), 0xb0);
    assert_eq!(size_of::<site::button_triggered::ButtonTriggered>(), 0x20);
    assert_eq!(size_of::<site::force_close::ForceClose>(), 0x08);
}

#[test]
fn sitehandler_offsets() {
    use site::update_visual::UpdateVisual;
    assert_eq!(offset_of!(UpdateVisual, vignette), 0x08);
    assert_eq!(offset_of!(UpdateVisual, lead), 0x50);
    assert_eq!(offset_of!(UpdateVisual, button_count), 0x98);
    assert_eq!(offset_of!(UpdateVisual, buttons), 0xa0);
    assert_eq!(offset_of!(site::update_layout::UpdateLayout, rect), 0x0c);
    assert_eq!(offset_of!(site::update_layout::UpdateLayout, user_size), 0x1c);
    assert_eq!(offset_of!(site::button_triggered::ButtonTriggered, entry_text), 0x10);
}

#[test]
fn menu_payload_sizes() {
    assert_eq!(size_of::<menu::update_visual::UpdateVisual>(), 0x68);
    assert_eq!(size_of::<menu::update_layout::UpdateLayout>(), 0x18);
    assert_eq!(size_of::<menu::button_triggered::ButtonTriggered>(), 0x18);
}

#[test]
fn menu_offsets() {
    use menu::update_visual::UpdateVisual;
    assert_eq!(offset_of!(UpdateVisual, representation), 0x10);
    assert_eq!(offset_of!(UpdateVisual, xbutton_count), 0x58);
    assert_eq!(offset_of!(UpdateVisual, xbuttons), 0x60);
    assert_eq!(offset_of!(menu::update_layout::UpdateLayout, menu_layout), 0x04);
    assert_eq!(offset_of!(menu::button_triggered::ButtonTriggered, entry_text), 0x08);
}

#[test]
fn dialog_payload_sizes() {
    use fprt_sys::ui::{favorites, inputfa, recentlyvisited as recents, AddressList, AddressSelection};
    assert_eq!(size_of::<AddressList>(), 0x18);
    assert_eq!(size_of::<AddressSelection>(), 0x18);
    assert_eq!(offset_of!(AddressList, count), 0x08);
    assert_eq!(offset_of!(AddressList, items), 0x10);
    assert_eq!(offset_of!(AddressSelection, items), 0x10);
    assert_eq!(size_of::<recents::labels::Labels>(), 0x68);
    assert_eq!(size_of::<favorites::labels::Labels>(), 0x68);
    assert_eq!(size_of::<inputfa::labels::Labels>(), 0x58);
    assert_eq!(size_of::<inputfa::update_address::UpdateAddress>(), 0x18);
    assert_eq!(size_of::<inputfa::update_error_raise::UpdateErrorRaise>(), 0x18);
    assert_eq!(size_of::<inputfa::field_text::FieldText>(), 0x18);
}
