//! Compile-and-run guard that the `#[repr(C)]` payloads match the reverse-
//! engineered byte layouts. Sizes/offsets come from the RE dossiers; a mismatch
//! here means a drop-in break.

use std::mem::{offset_of, size_of};

use fprt_sys::ui::application as app;
use fprt_sys::ui::{image_record::ImageRecord, layout_tuple::LayoutTuple, sld_rect::SldRect};

#[test]
fn shared_sizes() {
    assert_eq!(size_of::<ImageRecord>(), 0x18);
    assert_eq!(size_of::<SldRect>(), 0x10);
    assert_eq!(size_of::<LayoutTuple>(), 0x14);
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
