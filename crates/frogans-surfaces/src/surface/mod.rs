//! A [`Surface`] — one winit window bound to a conductor component instance,
//! rendered through egui on wgpu.
//!
//! Phase 2 is the unified skeleton: every surface clears its window and runs an
//! egui frame (a placeholder naming the bound [`SurfaceKey`]). Phase 3 splits this
//! into the engine-pixel *visual* path (site/menu/pad) and the egui *chrome* path
//! (the list dialogs), but they share everything here: the swap-chain surface, the
//! egui context/state, and the renderer.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;

use egui_wgpu::{Renderer, RendererOptions, ScreenDescriptor};
use fprt::menu::UpdateVisual as MenuVisual;
use fprt::sitehandler::UpdateVisual;
use fprt::visual::{Rect, Representation, Rollover};
use fprt::PooledImage;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::window::{Window, WindowId};

use crate::gpu::Gpu;
use crate::{SurfaceKey, WindowPosition};

/// Mask-only piece kind: the clickable silhouette (1-bpp `raw` plane, no color).
/// A zone's silhouette is the union of these; the colored pieces clip to it.
const KIND_HIT: i32 = 2001;

/// How far the pointer must move while pressed before it counts as a drag rather
/// than a click (physical pixels).
const DRAG_THRESHOLD: f64 = 4.0;

/// What a surface's input produced, for the session to act on.
pub enum SurfaceInput {
    /// An interactive zone was activated (clicked). Carries the 0-based zone /
    /// button index to report via `ReportButtonTriggered`.
    Activate(i32),
    /// The pad was clicked — the user wants the menu.
    MenuAccess,
    /// A chrome (egui) dialog produced an action.
    Chrome(ChromeAction),
    /// The address-entry (`inputfa`) dialog produced an action.
    Input(InputAction),
    /// The leap-confirmation (`leaptofrogans`) dialog produced an action.
    Confirm(ConfirmAction),
    /// The language dialog produced an action.
    Language(LanguageAction),
    /// The zoom dialog produced an action.
    Zoom(ZoomAction),
    /// The update dialog produced an action.
    Update(UpdateAction),
    /// The legal-information dialog produced an action.
    Legal(LegalAction),
    /// The inspector dialog produced an action.
    Inspector(InspectorAction),
    /// The window is being dragged — ask the embedder to move it.
    Reposition(WindowPosition),
}

/// An action from the `inspector` dialog.
pub enum InspectorAction {
    /// A run step was selected (its index).
    SelectStep(i32),
    /// A content view was selected (its index).
    SelectContent(i32),
    /// The autosync checkbox was toggled.
    Autosync(bool),
    /// Synchronize now.
    Synchronize,
    /// Re-run.
    Rerun,
    /// Close the inspector.
    Close,
}

/// An action from the `language` dialog.
pub enum LanguageAction {
    /// Choose a language (its identifier).
    Choose(String),
    /// Dismiss.
    Cancel,
}

/// An action from the `zoom` dialog.
pub enum ZoomAction {
    /// Use the engine's default scaling factor.
    Default,
    /// Apply a custom zoom (percent, engine clamps to 50..=200).
    Apply(i32),
    /// Dismiss.
    Cancel,
}

/// An action from the `update` dialog.
pub enum UpdateAction {
    /// Open one of the update URIs externally (a host service).
    OpenUri(String),
    /// Dismiss.
    Cancel,
}

/// An action from the `legalinformation` dialog.
pub enum LegalAction {
    /// Close the viewer.
    Close,
}

/// An action from the address-entry (`inputfa`) dialog.
pub enum InputAction {
    /// The field text changed (live; for engine validation).
    Change(String),
    /// Submit the entered address.
    Submit(String),
    /// Dismiss the dialog.
    Cancel,
}

/// An action from the leap-confirmation (`leaptofrogans`) dialog.
pub enum ConfirmAction {
    /// Confirm the leap.
    Confirm,
    /// Cancel.
    Cancel,
    /// Block the address.
    Block,
    /// Purge the address.
    Purge,
    /// Close the dialog.
    Close,
}

/// A press in progress on a visual/pad surface, tracked to tell a click from a
/// drag: a press that releases in place is a click, one that moves is a drag.
struct PressState {
    /// Where the button went down, in window-relative pixels (the drag grab point).
    grab: (f64, f64),
    /// Whether the pointer has crossed [`DRAG_THRESHOLD`] (became a drag).
    dragging: bool,
}

/// An action from a chrome address-list dialog. The session maps each to the
/// owning component's event (e.g. `Open` → `favorites::ReportOpen` /
/// `devtools::ReportInspect`).
pub enum ChromeAction {
    /// Primary action on a row (open / inspect a clicked address).
    Open(String),
    /// Remove / delete one address (its ✕ button).
    Remove(String),
    /// Remove / delete every address.
    RemoveAll,
    /// Dismiss the dialog.
    Cancel,
}

/// The non-address configuration of a chrome dialog: its title plus which actions
/// it offers (the session fills this in per component). Addresses arrive
/// separately via [`Surface::set_chrome_addresses`].
pub(crate) struct ChromeConfig {
    pub title: Option<String>,
    /// Whether a row click is meaningful (false for `blocked`, which only removes).
    pub can_open: bool,
    /// Whether each row gets a remove (✕) button.
    pub can_remove: bool,
    /// Label for the bottom "remove all" button (None ⇒ absent).
    pub remove_all: Option<String>,
    /// Label for the bottom cancel button.
    pub cancel: Option<String>,
}

/// What a surface draws.
enum Content {
    /// A rendered engine [`Representation`] — site slide or menu — with hover
    /// zones. Boxed to keep the enum small.
    Visual(Box<VisualContent>),
    /// The pad: a single engine image; clicking anywhere requests the menu.
    Pad(PadContent),
    /// An egui list dialog (e.g. recently-visited): a title, an address list, and
    /// a cancel button.
    Chrome(ChromeContent),
    /// The address-entry dialog (`inputfa`): a text field + ok/cancel.
    Input(Box<InputContent>),
    /// The leap-confirmation dialog (`leaptofrogans`): an address + action buttons.
    Confirm(Box<ConfirmContent>),
    /// The language picker: a selectable list + ok/cancel.
    Language(Box<LanguageContent>),
    /// The zoom dialog: a percent slider + default/ok/cancel.
    Zoom(ZoomContent),
    /// The update dialog: instruction text + download/branch links + cancel.
    Update(Box<UpdateContent>),
    /// The legal-information viewer: topic text + close.
    Legal(Box<LegalContent>),
    /// A per-site run inspector (multi-instance).
    Inspector(Box<InspectorContent>),
}

/// State for the `language` dialog. Each entry is `(identifier, display name)`.
#[derive(Default)]
struct LanguageContent {
    title: Option<String>,
    cancel: Option<String>,
    languages: Vec<(String, String)>,
    current: Option<String>,
}

/// State for the `zoom` dialog.
struct ZoomContent {
    title: Option<String>,
    default_label: Option<String>,
    ok: Option<String>,
    cancel: Option<String>,
    /// The slider's current percent (engine clamps 50..=200).
    percent: i32,
}

impl Default for ZoomContent {
    fn default() -> Self {
        ZoomContent {
            title: None,
            default_label: None,
            ok: None,
            cancel: None,
            percent: 100,
        }
    }
}

/// State for the `update` dialog.
#[derive(Default)]
struct UpdateContent {
    title: Option<String>,
    instruction: Option<String>,
    download: Option<String>,
    cancel: Option<String>,
    /// The update URI (opened by the download button).
    update_uri: Option<String>,
    /// The changed-branch URI (a secondary link).
    branch_uri: Option<String>,
}

/// State for the `legalinformation` viewer.
#[derive(Default)]
struct LegalContent {
    title: Option<String>,
    close: Option<String>,
    /// Flattened topics of the default-language document: `(title, text)`.
    topics: Vec<(String, String)>,
}

/// Labels for the `language` dialog (the session fills this).
pub(crate) struct LanguageConfig {
    pub title: Option<String>,
    pub cancel: Option<String>,
}

/// Labels for the `zoom` dialog (the session fills this).
pub(crate) struct ZoomConfig {
    pub title: Option<String>,
    pub default_label: Option<String>,
    pub ok: Option<String>,
    pub cancel: Option<String>,
}

/// Labels for the `update` dialog (the session fills this).
pub(crate) struct UpdateConfig {
    pub title: Option<String>,
    pub instruction: Option<String>,
    pub download: Option<String>,
    pub cancel: Option<String>,
}

/// Labels for the `legalinformation` dialog (the session fills this).
pub(crate) struct LegalConfig {
    pub title: Option<String>,
    pub close: Option<String>,
}

/// State for an `inspector` window (multi-instance, one per running site).
#[derive(Default)]
struct InspectorContent {
    title: Option<String>,
    address: Option<String>,
    /// Run status text (completed / rejection), already resolved by the session.
    status: Option<String>,
    /// Run steps (a combobox) + the active index.
    steps: Vec<String>,
    active_step: i32,
    /// Content views (a combobox) + the active index.
    content_labels: Vec<String>,
    active_content: i32,
    /// The selected content document text.
    content: String,
    autosync_on: bool,
    synchronize_enabled: bool,
    synchronize_label: Option<String>,
    rerun_label: Option<String>,
    close_label: Option<String>,
}

/// Labels + flags for an inspector window (the session fills this).
pub(crate) struct InspectorConfig {
    pub title: Option<String>,
    pub synchronize: Option<String>,
    pub rerun: Option<String>,
    pub close: Option<String>,
}

/// State for a [`Content::Input`] dialog (`inputfa`).
#[derive(Default)]
struct InputContent {
    title: Option<String>,
    instruction: Option<String>,
    placeholder: Option<String>,
    ok: Option<String>,
    cancel: Option<String>,
    /// Inline error (from `update_error_raise`; cleared by `update_error_clear`).
    error: Option<String>,
    /// The field's current text (engine-seeded via `update_address`, then edited).
    text: String,
}

/// Title + button labels of an [`Content::Input`] dialog (the session fills this).
pub(crate) struct InputConfig {
    pub title: Option<String>,
    pub instruction: Option<String>,
    pub placeholder: Option<String>,
    pub ok: Option<String>,
    pub cancel: Option<String>,
}

/// State for a [`Content::Confirm`] dialog (`leaptofrogans`).
#[derive(Default)]
struct ConfirmContent {
    title: Option<String>,
    instruction: Option<String>,
    /// The candidate address + whether it's compliant.
    address: Option<String>,
    compliant: bool,
    confirm: Option<String>,
    cancel: Option<String>,
    block: Option<String>,
    purge: Option<String>,
    close: Option<String>,
}

/// Title + button labels of a [`Content::Confirm`] dialog (the session fills this).
pub(crate) struct ConfirmConfig {
    pub title: Option<String>,
    pub instruction: Option<String>,
    pub confirm: Option<String>,
    pub cancel: Option<String>,
    pub block: Option<String>,
    pub purge: Option<String>,
    pub close: Option<String>,
}

/// State for a [`Content::Chrome`] dialog.
#[derive(Default)]
struct ChromeContent {
    /// Title + which actions the dialog offers.
    config: Option<ChromeConfig>,
    /// The addresses shown as rows.
    addresses: Vec<String>,
}

impl Default for ChromeConfig {
    fn default() -> Self {
        ChromeConfig {
            title: None,
            can_open: true,
            can_remove: false,
            remove_all: None,
            cancel: Some("Cancel".to_string()),
        }
    }
}

/// State for a [`Content::Visual`] surface (site or menu).
#[derive(Default)]
struct VisualContent {
    /// The resting representation image, uploaded as an egui texture.
    base: Option<egui::TextureHandle>,
    /// Representation image size in engine pixels (for the cursor↔slide mapping).
    slide_size: [u32; 2],
    /// The representation's interactive zones, parallel to its `rollovers` (so a
    /// zone's index *is* its button index). Pre-uploaded hover pieces included.
    zones: Vec<Zone>,
    /// The zone the cursor is currently over, if any.
    hovered: Option<usize>,
    /// The in-progress (loading) spinner, shown over the slide while running.
    spinner: Option<Animation>,
}

/// State for a [`Content::Pad`] surface.
#[derive(Default)]
struct PadContent {
    /// The pad icon (`ApplicationUpdateImages.pad_main`), as an egui texture.
    image: Option<egui::TextureHandle>,
    /// Pad image size in engine pixels.
    size: [u32; 2],
    /// The active loading animation, if any (replaces the icon while running).
    anim: Option<Animation>,
}

/// A frame-cycling animation: uploaded frames played at a fixed inter-frame delay.
struct Animation {
    frames: Vec<egui::TextureHandle>,
    /// Frame dimensions in engine pixels (uniform across frames).
    size: [u32; 2],
    delay_ms: u64,
    start: Instant,
}

impl Animation {
    /// The frame to show now, picked by elapsed time. `None` if empty.
    fn current(&self) -> Option<egui::TextureId> {
        if self.frames.is_empty() {
            return None;
        }
        let ticks = (self.start.elapsed().as_millis() as u64)
            .checked_div(self.delay_ms)
            .unwrap_or(0);
        Some(self.frames[ticks as usize % self.frames.len()].id())
    }
}

/// One interactive zone: its clickable silhouette plus the textures painted over
/// the resting slide while it's hovered (the engine's hover-state pieces).
struct Zone {
    /// The clickable shape: the slide-pixel coordinates inside the union of the
    /// zone's 2001 masks. Hit-testing uses this real shape, not a bounding box, so
    /// hover/click fire only on the actual pill/badge/arrow.
    silhouette: HashSet<(i32, i32)>,
    /// Fallback rectangle for a zone with no 2001 mask (empty silhouette).
    region: Rect,
    /// Hover pieces: each `(geom, texture)` painted in array (paint) order.
    pieces: Vec<(Rect, egui::TextureHandle)>,
}

/// An owned per-frame paint description, so the egui closure needn't borrow
/// `self`: the resting slide plus the hovered zone's overlay pieces.
struct PaintFrame {
    base: egui::TextureId,
    /// Slide size in engine pixels, for mapping piece geometry into the window.
    slide: [f32; 2],
    /// `(texture, slide-pixel geom)` for the hovered zone's pieces (paint order).
    overlay: Vec<(egui::TextureId, Rect)>,
}

/// An owned snapshot of a chrome dialog for the egui closure.
struct ChromeView {
    title: String,
    addresses: Vec<String>,
    can_open: bool,
    can_remove: bool,
    remove_all: Option<String>,
    cancel: Option<String>,
}

/// An owned snapshot of the leap-confirmation dialog for the egui closure.
struct ConfirmView {
    title: String,
    instruction: String,
    address: String,
    compliant: bool,
    confirm: Option<String>,
    cancel: Option<String>,
    block: Option<String>,
    purge: Option<String>,
    close: Option<String>,
}

/// An owned snapshot of the address-entry dialog. `text` is editable in-frame and
/// written back after.
struct InputView {
    title: String,
    instruction: String,
    placeholder: String,
    ok: Option<String>,
    cancel: Option<String>,
    error: Option<String>,
    text: String,
}

/// Snapshot of the `language` dialog.
struct LanguageView {
    title: String,
    cancel: Option<String>,
    languages: Vec<(String, String)>,
    current: Option<String>,
}

/// Snapshot of the `zoom` dialog. `percent` is editable in-frame and written back.
struct ZoomView {
    title: String,
    default_label: Option<String>,
    ok: Option<String>,
    cancel: Option<String>,
    percent: i32,
}

/// Snapshot of the `update` dialog.
struct UpdateView {
    title: String,
    instruction: String,
    download: Option<String>,
    cancel: Option<String>,
    update_uri: Option<String>,
    branch_uri: Option<String>,
}

/// Snapshot of the `legalinformation` viewer.
struct LegalView {
    title: String,
    close: Option<String>,
    topics: Vec<(String, String)>,
}

/// Snapshot of an `inspector` window. The combobox/checkbox fields are editable
/// in-frame and written back after.
struct InspectorView {
    title: String,
    address: String,
    status: String,
    steps: Vec<String>,
    active_step: i32,
    content_labels: Vec<String>,
    active_content: i32,
    content: String,
    autosync_on: bool,
    synchronize_enabled: bool,
    synchronize_label: Option<String>,
    rerun_label: Option<String>,
    close_label: Option<String>,
}

/// One window + its egui/wgpu paint state, bound to a component instance.
pub struct Surface {
    key: SurfaceKey,
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    renderer: Renderer,
    content: Content,
    /// Last known physical cursor position (window-relative), for hover + drag.
    cursor: Option<(f64, f64)>,
    /// A press in progress (visual/pad surfaces), for click-vs-drag resolution.
    press: Option<PressState>,
    /// Whether the window is transparent (engine-visual surfaces) — clear the
    /// frame transparent so the slide/pad alpha shows, vs an opaque dialog backdrop.
    transparent: bool,
}

impl Surface {
    /// Bind a freshly created `window` to `key`, configuring its swap-chain
    /// surface and egui plumbing. Requests an initial redraw.
    pub fn new(key: SurfaceKey, window: Arc<Window>, gpu: &mut Gpu) -> Self {
        // The engine-visual windows (site / menu / pad) are transparent so the
        // slide/pad alpha shows; the dialogs are opaque.
        let transparent = matches!(
            key,
            SurfaceKey::Site(_) | SurfaceKey::Menu | SurfaceKey::Pad
        );
        let (surface, config) = gpu.configure_window(&window, transparent);

        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &*window,
            Some(window.scale_factor() as f32),
            window.theme(),
            None,
        );
        let renderer = Renderer::new(gpu.device(), config.format, RendererOptions::default());

        // Site + menu render an engine representation; the pad renders an image;
        // the address-list dialogs are egui chrome.
        let content = match key {
            SurfaceKey::Site(_) | SurfaceKey::Menu => Content::Visual(Box::default()),
            SurfaceKey::Pad => Content::Pad(PadContent::default()),
            SurfaceKey::Recent
            | SurfaceKey::Favorites
            | SurfaceKey::Blocked
            | SurfaceKey::Devtools
            | SurfaceKey::Recovery => Content::Chrome(ChromeContent::default()),
            SurfaceKey::Inputfa => Content::Input(Box::default()),
            SurfaceKey::Leaptofrogans => Content::Confirm(Box::default()),
            SurfaceKey::Language => Content::Language(Box::default()),
            SurfaceKey::Zoom => Content::Zoom(ZoomContent::default()),
            SurfaceKey::Update => Content::Update(Box::default()),
            SurfaceKey::Legal => Content::Legal(Box::default()),
            SurfaceKey::Inspector(_) => Content::Inspector(Box::default()),
        };

        window.request_redraw();
        Surface {
            key,
            window,
            surface,
            config,
            egui_ctx,
            egui_state,
            renderer,
            content,
            cursor: None,
            press: None,
            transparent,
        }
    }

    /// The backing window (for the embedder's geometry callbacks).
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Render a Frogans Site's lead slide. Returns the content size (for the
    /// embedder to size the window to), if any.
    pub fn set_site_visual(&mut self, visual: UpdateVisual) -> Option<(u32, u32)> {
        self.set_representation(&visual.lead)
    }

    /// Render the menu's representation. Returns the content size, if any.
    pub fn set_menu_visual(&mut self, visual: MenuVisual) -> Option<(u32, u32)> {
        self.set_representation(&visual.representation)
    }

    /// Adopt a new engine [`Representation`] (site slide or menu): upload the
    /// resting image and each zone's silhouette-clipped hover pieces, rebuild the
    /// zone table, and re-apply hover. Returns the content size (the embedder sizes
    /// the window).
    fn set_representation(&mut self, rep: &Representation) -> Option<(u32, u32)> {
        if !matches!(self.content, Content::Visual(_)) {
            return None;
        }

        // Upload against the egui context first (borrows `self.egui_ctx`), then
        // move the results into `content` (borrows `self.content`).
        let base = rep
            .image
            .as_ref()
            .map(|image| self.upload(image.bytes(), image.width(), image.height()));
        let slide_size = rep
            .image
            .as_ref()
            .map(|i| [i.width(), i.height()])
            .unwrap_or_default();

        let zones: Vec<Zone> = rep
            .rollovers
            .iter()
            .map(|rollover| {
                // A zone's hover appearance is its encoded pieces (the 2002 pill /
                // 2003 content layers); the 1-bpp `raw`-only pieces are hit masks.
                // Each colored piece is clipped to the zone silhouette (union of
                // the 2001 masks), else its opaque-rectangle background bleeds past
                // the real pill/arrow shape.
                let silhouette = zone_silhouette(rollover);
                let pieces = rollover
                    .pieces
                    .iter()
                    .filter_map(|piece| {
                        let image = piece.encoded.as_ref()?;
                        Some((piece.geom, self.upload_piece(image, piece.geom, &silhouette)))
                    })
                    .collect();
                Zone {
                    silhouette,
                    region: rollover.region,
                    pieces,
                }
            })
            .collect();

        {
            let Content::Visual(content) = &mut self.content else {
                return None;
            };
            content.base = base;
            content.slide_size = slide_size;
            content.zones = zones;
        }
        // Re-apply hover at the unchanged cursor, so a click that navigates to a
        // new page shows the hover state immediately (no mouse move required).
        self.refresh_hover();
        self.window.request_redraw();
        (slide_size != [0, 0]).then_some((slide_size[0], slide_size[1]))
    }

    /// Set the pad icon image. Returns the content size (the embedder sizes the
    /// window).
    pub fn set_pad_image(&mut self, image: &PooledImage) -> Option<(u32, u32)> {
        if !matches!(self.content, Content::Pad(_)) {
            return None;
        }
        let texture = self.upload(image.bytes(), image.width(), image.height());
        if let Content::Pad(pad) = &mut self.content {
            pad.image = Some(texture);
            pad.size = [image.width(), image.height()];
        }
        self.window.request_redraw();
        Some((image.width(), image.height()))
    }

    /// Build an [`Animation`] by uploading `frames` (each `(rgba, width, height)`).
    fn make_animation(&self, frames: &[(&[u8], u32, u32)], delay_ms: u64) -> Animation {
        let size = frames.first().map_or([0, 0], |(_, w, h)| [*w, *h]);
        Animation {
            frames: frames.iter().map(|(b, w, h)| self.upload(b, *w, *h)).collect(),
            size,
            delay_ms,
            start: Instant::now(),
        }
    }

    /// Begin the pad loading animation (frames replace the icon while running).
    pub fn start_pad_animation(&mut self, frames: &[(&[u8], u32, u32)], delay_ms: u64) {
        if !matches!(self.content, Content::Pad(_)) {
            return;
        }
        let anim = self.make_animation(frames, delay_ms);
        if let Content::Pad(p) = &mut self.content {
            p.anim = Some(anim);
        }
        self.window.request_redraw();
    }

    /// End the pad loading animation (revert to the static icon).
    pub fn stop_pad_animation(&mut self) {
        if let Content::Pad(p) = &mut self.content {
            p.anim = None;
            self.window.request_redraw();
        }
    }

    /// Begin the site in-progress spinner (drawn over the slide).
    pub fn start_spinner(&mut self, frames: &[(&[u8], u32, u32)], delay_ms: u64) {
        if !matches!(self.content, Content::Visual(_)) {
            return;
        }
        let anim = self.make_animation(frames, delay_ms);
        if let Content::Visual(v) = &mut self.content {
            v.spinner = Some(anim);
        }
        self.window.request_redraw();
    }

    /// End the site in-progress spinner.
    pub fn stop_spinner(&mut self) {
        if let Content::Visual(v) = &mut self.content {
            v.spinner = None;
            self.window.request_redraw();
        }
    }

    /// Whether an animation is running (so `redraw` keeps requesting frames).
    fn animating(&self) -> bool {
        match &self.content {
            Content::Pad(p) => p.anim.is_some(),
            Content::Visual(v) => v.spinner.is_some(),
            _ => false,
        }
    }

    /// Set a chrome dialog's title + available actions (per component).
    pub fn set_chrome_config(&mut self, config: ChromeConfig) {
        if let Content::Chrome(chrome) = &mut self.content {
            chrome.config = Some(config);
            self.window.request_redraw();
        }
    }

    /// Set a chrome dialog's address rows.
    pub fn set_chrome_addresses(&mut self, addresses: Vec<String>) {
        if let Content::Chrome(chrome) = &mut self.content {
            chrome.addresses = addresses;
            self.window.request_redraw();
        }
    }

    /// Set the address-entry dialog's title + button labels.
    pub fn set_input_config(&mut self, config: InputConfig) {
        if let Content::Input(c) = &mut self.content {
            c.title = config.title;
            c.instruction = config.instruction;
            c.placeholder = config.placeholder;
            c.ok = config.ok;
            c.cancel = config.cancel;
            self.window.request_redraw();
        }
    }

    /// Seed the address-entry field's text (engine `update_address`).
    pub fn set_input_text(&mut self, text: String) {
        if let Content::Input(c) = &mut self.content {
            c.text = text;
            self.window.request_redraw();
        }
    }

    /// Set / clear the address-entry dialog's inline error.
    pub fn set_input_error(&mut self, error: Option<String>) {
        if let Content::Input(c) = &mut self.content {
            c.error = error;
            self.window.request_redraw();
        }
    }

    /// Set the leap-confirmation dialog's title + button labels.
    pub fn set_confirm_config(&mut self, config: ConfirmConfig) {
        if let Content::Confirm(c) = &mut self.content {
            c.title = config.title;
            c.instruction = config.instruction;
            c.confirm = config.confirm;
            c.cancel = config.cancel;
            c.block = config.block;
            c.purge = config.purge;
            c.close = config.close;
            self.window.request_redraw();
        }
    }

    /// Set the leap-confirmation dialog's candidate address + compliance.
    pub fn set_confirm_address(&mut self, address: Option<String>, compliant: bool) {
        if let Content::Confirm(c) = &mut self.content {
            c.address = address;
            c.compliant = compliant;
            self.window.request_redraw();
        }
    }

    /// Set the language dialog's labels.
    pub fn set_language_config(&mut self, config: LanguageConfig) {
        if let Content::Language(c) = &mut self.content {
            c.title = config.title;
            c.cancel = config.cancel;
            self.window.request_redraw();
        }
    }

    /// Set the language dialog's selectable list + current selection.
    pub fn set_language_list(&mut self, languages: Vec<(String, String)>, current: Option<String>) {
        if let Content::Language(c) = &mut self.content {
            c.languages = languages;
            c.current = current;
            self.window.request_redraw();
        }
    }

    /// Set the zoom dialog's labels.
    pub fn set_zoom_config(&mut self, config: ZoomConfig) {
        if let Content::Zoom(c) = &mut self.content {
            c.title = config.title;
            c.default_label = config.default_label;
            c.ok = config.ok;
            c.cancel = config.cancel;
            self.window.request_redraw();
        }
    }

    /// Seed the zoom dialog's slider with the current percent (engine `update_zoom`).
    pub fn set_zoom_percent(&mut self, percent: i32) {
        if let Content::Zoom(c) = &mut self.content {
            c.percent = percent;
            self.window.request_redraw();
        }
    }

    /// Set the update dialog's labels.
    pub fn set_update_config(&mut self, config: UpdateConfig) {
        if let Content::Update(c) = &mut self.content {
            c.title = config.title;
            c.instruction = config.instruction;
            c.download = config.download;
            c.cancel = config.cancel;
            self.window.request_redraw();
        }
    }

    /// Set the update dialog's URIs (download target + changed-branch link).
    pub fn set_update_uris(&mut self, update_uri: Option<String>, branch_uri: Option<String>) {
        if let Content::Update(c) = &mut self.content {
            c.update_uri = update_uri;
            c.branch_uri = branch_uri;
            self.window.request_redraw();
        }
    }

    /// Set the legal-information viewer's labels.
    pub fn set_legal_config(&mut self, config: LegalConfig) {
        if let Content::Legal(c) = &mut self.content {
            c.title = config.title;
            c.close = config.close;
            self.window.request_redraw();
        }
    }

    /// Set the legal-information viewer's topics (`(title, text)`).
    pub fn set_legal_topics(&mut self, topics: Vec<(String, String)>) {
        if let Content::Legal(c) = &mut self.content {
            c.topics = topics;
            self.window.request_redraw();
        }
    }

    /// Set an inspector window's labels (title + button labels).
    pub fn set_inspector_config(&mut self, config: InspectorConfig) {
        if let Content::Inspector(c) = &mut self.content {
            c.title = config.title;
            c.synchronize_label = config.synchronize;
            c.rerun_label = config.rerun;
            c.close_label = config.close;
            self.window.request_redraw();
        }
    }

    /// Set an inspector window's inspected address.
    pub fn set_inspector_address(&mut self, address: Option<String>) {
        if let Content::Inspector(c) = &mut self.content {
            c.address = address;
            self.window.request_redraw();
        }
    }

    /// Set an inspector window's run-status text.
    pub fn set_inspector_status(&mut self, status: Option<String>) {
        if let Content::Inspector(c) = &mut self.content {
            c.status = status;
            self.window.request_redraw();
        }
    }

    /// Set an inspector window's run steps + active index.
    pub fn set_inspector_steps(&mut self, steps: Vec<String>, active: i32) {
        if let Content::Inspector(c) = &mut self.content {
            c.steps = steps;
            c.active_step = active;
            self.window.request_redraw();
        }
    }

    /// Set an inspector window's content views + active index.
    pub fn set_inspector_content_labels(&mut self, labels: Vec<String>, active: i32) {
        if let Content::Inspector(c) = &mut self.content {
            c.content_labels = labels;
            c.active_content = active;
            self.window.request_redraw();
        }
    }

    /// Set an inspector window's content document text.
    pub fn set_inspector_content(&mut self, content: String) {
        if let Content::Inspector(c) = &mut self.content {
            c.content = content;
            self.window.request_redraw();
        }
    }

    /// Set an inspector window's autosync state.
    pub fn set_inspector_sync(&mut self, autosync_on: bool, synchronize_enabled: bool) {
        if let Content::Inspector(c) = &mut self.content {
            c.autosync_on = autosync_on;
            c.synchronize_enabled = synchronize_enabled;
            self.window.request_redraw();
        }
    }

    /// Snapshot a chrome dialog's content for the egui closure (so it needn't
    /// borrow `self`).
    fn chrome_view(&self) -> Option<ChromeView> {
        let Content::Chrome(c) = &self.content else {
            return None;
        };
        let config = c.config.as_ref();
        Some(ChromeView {
            title: config.and_then(|c| c.title.clone()).unwrap_or_default(),
            can_open: config.is_none_or(|c| c.can_open),
            can_remove: config.is_some_and(|c| c.can_remove),
            remove_all: config.and_then(|c| c.remove_all.clone()),
            cancel: config
                .and_then(|c| c.cancel.clone())
                .or_else(|| Some("Cancel".to_string())),
            addresses: c.addresses.clone(),
        })
    }

    /// Snapshot the leap-confirmation dialog for the egui closure.
    fn confirm_view(&self) -> Option<ConfirmView> {
        let Content::Confirm(c) = &self.content else {
            return None;
        };
        Some(ConfirmView {
            title: c.title.clone().unwrap_or_default(),
            instruction: c.instruction.clone().unwrap_or_default(),
            address: c.address.clone().unwrap_or_default(),
            compliant: c.compliant,
            confirm: c.confirm.clone(),
            cancel: c.cancel.clone(),
            block: c.block.clone(),
            purge: c.purge.clone(),
            close: c.close.clone(),
        })
    }

    /// Snapshot the address-entry dialog for the egui closure.
    fn input_view(&self) -> Option<InputView> {
        let Content::Input(c) = &self.content else {
            return None;
        };
        Some(InputView {
            title: c.title.clone().unwrap_or_default(),
            instruction: c.instruction.clone().unwrap_or_default(),
            placeholder: c.placeholder.clone().unwrap_or_default(),
            ok: c.ok.clone(),
            cancel: c.cancel.clone(),
            error: c.error.clone(),
            text: c.text.clone(),
        })
    }

    /// Snapshot the `language` dialog.
    fn language_view(&self) -> Option<LanguageView> {
        let Content::Language(c) = &self.content else {
            return None;
        };
        Some(LanguageView {
            title: c.title.clone().unwrap_or_default(),
            cancel: c.cancel.clone(),
            languages: c.languages.clone(),
            current: c.current.clone(),
        })
    }

    /// Snapshot the `zoom` dialog.
    fn zoom_view(&self) -> Option<ZoomView> {
        let Content::Zoom(c) = &self.content else {
            return None;
        };
        Some(ZoomView {
            title: c.title.clone().unwrap_or_default(),
            default_label: c.default_label.clone(),
            ok: c.ok.clone(),
            cancel: c.cancel.clone(),
            percent: c.percent,
        })
    }

    /// Snapshot the `update` dialog.
    fn update_view(&self) -> Option<UpdateView> {
        let Content::Update(c) = &self.content else {
            return None;
        };
        Some(UpdateView {
            title: c.title.clone().unwrap_or_default(),
            instruction: c.instruction.clone().unwrap_or_default(),
            download: c.download.clone(),
            cancel: c.cancel.clone(),
            update_uri: c.update_uri.clone(),
            branch_uri: c.branch_uri.clone(),
        })
    }

    /// Snapshot the `legalinformation` viewer.
    fn legal_view(&self) -> Option<LegalView> {
        let Content::Legal(c) = &self.content else {
            return None;
        };
        Some(LegalView {
            title: c.title.clone().unwrap_or_default(),
            close: c.close.clone(),
            topics: c.topics.clone(),
        })
    }

    /// Snapshot an inspector window for the egui closure.
    fn inspector_view(&self) -> Option<InspectorView> {
        let Content::Inspector(c) = &self.content else {
            return None;
        };
        Some(InspectorView {
            title: c.title.clone().unwrap_or_default(),
            address: c.address.clone().unwrap_or_default(),
            status: c.status.clone().unwrap_or_default(),
            steps: c.steps.clone(),
            active_step: c.active_step,
            content_labels: c.content_labels.clone(),
            active_content: c.active_content,
            content: c.content.clone(),
            autosync_on: c.autosync_on,
            synchronize_enabled: c.synchronize_enabled,
            synchronize_label: c.synchronize_label.clone(),
            rerun_label: c.rerun_label.clone(),
            close_label: c.close_label.clone(),
        })
    }

    /// Upload a straight-alpha RGBA buffer as a nearest-filtered egui texture.
    fn upload(&self, rgba: &[u8], width: u32, height: u32) -> egui::TextureHandle {
        let color =
            egui::ColorImage::from_rgba_unmultiplied([width as usize, height as usize], rgba);
        self.egui_ctx
            .load_texture("site", color, egui::TextureOptions::NEAREST)
    }

    /// Upload one hover piece, clipped to `silhouette`: pixels whose slide
    /// coordinate isn't inside the zone silhouette are made transparent, so the
    /// piece's opaque rectangle background doesn't bleed past the real shape.
    fn upload_piece(
        &self,
        image: &PooledImage,
        geom: Rect,
        silhouette: &HashSet<(i32, i32)>,
    ) -> egui::TextureHandle {
        let (w, h) = (image.width() as usize, image.height() as usize);
        let mut color = egui::ColorImage::from_rgba_unmultiplied([w, h], image.bytes());
        // No silhouette (a zone with no 2001 mask) ⇒ paint the piece unclipped.
        if !silhouette.is_empty() {
            for py in 0..h {
                for px in 0..w {
                    if !silhouette.contains(&(geom.x + px as i32, geom.y + py as i32)) {
                        color.pixels[py * w + px] = egui::Color32::TRANSPARENT;
                    }
                }
            }
        }
        self.egui_ctx
            .load_texture("site-piece", color, egui::TextureOptions::NEAREST)
    }

    /// The id of the backing window (for the session's `WindowId` → surface map).
    pub fn window_id(&self) -> WindowId {
        self.window.id()
    }

    /// Show or hide the window (engine `Show`/`Hide`). Showing requests a redraw.
    pub fn set_visible(&self, visible: bool) {
        self.window.set_visible(visible);
        if visible {
            self.window.request_redraw();
        }
    }

    /// Raise the window to the front (engine `Push`).
    pub fn raise(&self) {
        self.window.set_visible(true);
        self.window.focus_window();
        self.window.request_redraw();
    }

    /// Feed a window event to the surface. Forwards it to egui, then — for a
    /// visual/pad surface — runs the click-vs-drag machine: a press that releases
    /// in place is a click (zone activation / menu access); one that moves past the
    /// threshold is a drag (a stream of [`SurfaceInput::Reposition`]s). Hover is
    /// purely local; only activation and drag cross back out.
    ///
    /// Chrome dialogs are egui-driven: their actions surface from [`redraw`] and
    /// their window is dragged by the WM (a decorated dialog), so the machine here
    /// is skipped for them.
    ///
    /// [`redraw`]: Surface::redraw
    pub fn on_window_event(&mut self, event: &WindowEvent) -> Option<SurfaceInput> {
        // egui may need to repaint to process this event (a chrome dialog's button
        // click is only seen on the next frame), so honor its repaint request.
        let response = self.egui_state.on_window_event(&self.window, event);
        if response.repaint {
            self.window.request_redraw();
        }
        // Only the engine-pixel surfaces run the click/drag machine; the egui
        // dialogs are driven by egui and surface their actions from `redraw`.
        if !matches!(self.content, Content::Visual(_) | Content::Pad(_)) {
            return None;
        }

        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let pos = (position.x, position.y);
                self.cursor = Some(pos);
                if let Some(press) = &mut self.press {
                    let moved = (pos.0 - press.grab.0).hypot(pos.1 - press.grab.1);
                    if press.dragging || moved > DRAG_THRESHOLD {
                        press.dragging = true;
                        // Window-relative offset from the grab point; the embedder
                        // applies it to the window's current position (see
                        // `WindowPosition::Relative`).
                        return Some(SurfaceInput::Reposition(WindowPosition::Relative(
                            (pos.0 - press.grab.0).round() as i32,
                            (pos.1 - press.grab.1).round() as i32,
                        )));
                    }
                    None
                } else {
                    self.refresh_hover();
                    None
                }
            }
            WindowEvent::CursorLeft { .. } => {
                self.cursor = None;
                self.refresh_hover();
                None
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                // Don't act yet — the click fires on release if we don't drag.
                if let Some(grab) = self.cursor {
                    self.press = Some(PressState {
                        grab,
                        dragging: false,
                    });
                }
                None
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => match self.press.take() {
                // Released in place ⇒ a click.
                Some(press) if !press.dragging => self.click_action(),
                _ => None,
            },
            _ => None,
        }
    }

    /// The engine event a click (a press released in place) produces.
    fn click_action(&self) -> Option<SurfaceInput> {
        match &self.content {
            // The pad has no zones — a click anywhere requests the menu.
            Content::Pad(_) => Some(SurfaceInput::MenuAccess),
            Content::Visual(v) => v.hovered.map(|i| SurfaceInput::Activate(i as i32)),
            _ => None,
        }
    }

    /// The zone (in slide-pixel space) under a physical cursor position.
    fn locate(&self, cursor: (f64, f64)) -> Option<usize> {
        let Content::Visual(v) = &self.content else {
            return None;
        };
        let (sx, sy) =
            cursor_to_slide(cursor, v.slide_size, (self.config.width, self.config.height))?;
        // Test the real silhouette (the pixel under the cursor); fall back to the
        // bounding rect only for a zone that has no 2001 mask.
        let pixel = (sx.floor() as i32, sy.floor() as i32);
        v.zones.iter().position(|z| {
            if z.silhouette.is_empty() {
                rect_contains(z.region, sx, sy)
            } else {
                z.silhouette.contains(&pixel)
            }
        })
    }

    /// Recompute the hovered zone from the last known cursor position, redrawing on
    /// a change. Called on cursor moves and whenever the zones change underneath a
    /// stationary cursor (a new visual, or a resize that shifts the mapping).
    fn refresh_hover(&mut self) {
        let located = self.cursor.and_then(|c| self.locate(c));
        if let Content::Visual(v) = &mut self.content
            && v.hovered != located
        {
            v.hovered = located;
            self.window.request_redraw();
        }
    }

    /// Reconfigure the swap chain after a resize.
    pub fn resize(&mut self, gpu: &Gpu, size: PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(gpu.device(), &self.config);
            // The cursor→slide mapping depends on the window size, so re-apply hover.
            self.refresh_hover();
            self.window.request_redraw();
        }
    }

    /// Snapshot what this frame should paint (resting image + hovered overlay).
    fn paint_frame(&self) -> Option<PaintFrame> {
        match &self.content {
            Content::Visual(v) => {
                let base = v.base.as_ref()?.id();
                let mut overlay: Vec<(egui::TextureId, Rect)> = v
                    .hovered
                    .and_then(|i| v.zones.get(i))
                    .map(|zone| zone.pieces.iter().map(|(geom, tex)| (tex.id(), *geom)).collect())
                    .unwrap_or_default();
                // The in-progress spinner sits over the slide, top-left.
                if let Some(spinner) = &v.spinner
                    && let Some(id) = spinner.current()
                {
                    overlay.push((
                        id,
                        Rect { x: 8, y: 8, width: spinner.size[0] as i32, height: spinner.size[1] as i32 },
                    ));
                }
                Some(PaintFrame {
                    base,
                    slide: [v.slide_size[0] as f32, v.slide_size[1] as f32],
                    overlay,
                })
            }
            Content::Pad(p) => {
                // While loading, the animation frame replaces the static icon.
                let base = p
                    .anim
                    .as_ref()
                    .and_then(Animation::current)
                    .or_else(|| p.image.as_ref().map(egui::TextureHandle::id))?;
                Some(PaintFrame {
                    base,
                    slide: [p.size[0] as f32, p.size[1] as f32],
                    overlay: Vec::new(),
                })
            }
            // egui dialogs paint no engine image.
            _ => None,
        }
    }

    /// Render one egui frame: clear, run the UI, and present. Returns any input
    /// the frame produced (a chrome dialog's button click).
    pub fn redraw(&mut self, gpu: &Gpu) -> Option<SurfaceInput> {
        let device = gpu.device();
        let queue = gpu.queue();

        let raw_input = self.egui_state.take_egui_input(&self.window);
        let key = self.key;
        // Pull what the UI needs out of `content` first, so the closure doesn't
        // capture `self`.
        let frame = self.paint_frame();
        let chrome = self.chrome_view();
        let confirm = self.confirm_view();
        let mut input = self.input_view();
        let language = self.language_view();
        let mut zoom = self.zoom_view();
        let update = self.update_view();
        let legal = self.legal_view();
        let mut inspector = self.inspector_view();
        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        let mut action: Option<SurfaceInput> = None;
        let full_output = self.egui_ctx.run_ui(raw_input, |ui| {
            if let Some(paint) = &frame {
                let rect = ui.max_rect();
                ui.painter().image(paint.base, rect, uv, egui::Color32::WHITE);
                // Overlay the hovered zone's pieces, mapping slide pixels → window.
                let sx = rect.width() / paint.slide[0];
                let sy = rect.height() / paint.slide[1];
                for (texture, geom) in &paint.overlay {
                    let piece = egui::Rect::from_min_size(
                        egui::pos2(rect.min.x + geom.x as f32 * sx, rect.min.y + geom.y as f32 * sy),
                        egui::vec2(geom.width as f32 * sx, geom.height as f32 * sy),
                    );
                    ui.painter().image(*texture, piece, uv, egui::Color32::WHITE);
                }
            } else if let Some(view) = &chrome {
                action = chrome_ui(ui, view);
            } else if let Some(view) = &confirm {
                action = confirm_ui(ui, view);
            } else if let Some(view) = &mut input {
                action = input_ui(ui, view);
            } else if let Some(view) = &language {
                action = language_ui(ui, view);
            } else if let Some(view) = &mut zoom {
                action = zoom_ui(ui, view);
            } else if let Some(view) = &update {
                action = update_ui(ui, view);
            } else if let Some(view) = &legal {
                action = legal_ui(ui, view);
            } else if let Some(view) = &mut inspector {
                action = inspector_ui(ui, view);
            } else {
                ui.heading("frogans-surfaces");
                ui.label(format!("{key:?}"));
            }
        });

        // Persist any in-frame edits (the address field's text, the zoom slider).
        if let (Some(view), Content::Input(content)) = (&input, &mut self.content) {
            content.text = view.text.clone();
        }
        if let (Some(view), Content::Zoom(content)) = (&zoom, &mut self.content) {
            content.percent = view.percent;
        }
        // Optimistically persist inspector combobox/checkbox edits (the engine
        // sends the authoritative values back after the reported event).
        if let (Some(view), Content::Inspector(content)) = (&inspector, &mut self.content) {
            content.active_step = view.active_step;
            content.active_content = view.active_content;
            content.autosync_on = view.autosync_on;
        }

        self.egui_state
            .handle_platform_output(&self.window, full_output.platform_output);

        let primitives = self
            .egui_ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        for (id, delta) in &full_output.textures_delta.set {
            self.renderer.update_texture(device, queue, *id, delta);
        }

        let screen = ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: full_output.pixels_per_point,
        };

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frogans-surface-encoder"),
            });
        let prepare = self
            .renderer
            .update_buffers(device, queue, &mut encoder, &primitives, &screen);

        // Acquire the frame after preparing buffers; skip the frame if the
        // swap-chain isn't presentable (timeout / occluded / outdated).
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            _ => return None,
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("frogans-surface-pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            // Transparent surfaces clear to nothing so the engine
                            // image's alpha shows through; dialogs get a backdrop.
                            load: wgpu::LoadOp::Clear(if self.transparent {
                                wgpu::Color::TRANSPARENT
                            } else {
                                wgpu::Color { r: 0.05, g: 0.05, b: 0.07, a: 1.0 }
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                })
                .forget_lifetime();
            self.renderer.render(&mut pass, &primitives, &screen);
        }

        queue.submit(prepare.into_iter().chain(std::iter::once(encoder.finish())));
        frame.present();

        for id in &full_output.textures_delta.free {
            self.renderer.free_texture(id);
        }

        // Keep the frames flowing while an animation is running.
        if self.animating() {
            self.window.request_redraw();
        }

        action
    }
}

/// Render a chrome address-list dialog into `ui`, returning an action if a row or
/// a button was clicked. The action threads out through egui's `InnerResponse`
/// values, so the nested closures don't all fight over one `&mut`.
fn chrome_ui(ui: &mut egui::Ui, view: &ChromeView) -> Option<SurfaceInput> {
    if !view.title.is_empty() {
        ui.heading(&view.title);
    }
    ui.separator();

    let mut action = egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let mut row_action = None;
            for address in &view.addresses {
                // A per-row remove (✕), then the address — clickable to open where
                // that's meaningful, otherwise a plain label.
                let hit = ui
                    .horizontal(|ui| {
                        // `×` (U+00D7) — in Ubuntu-Light, unlike `✕` (U+2715),
                        // which egui's bundled font renders as tofu.
                        if view.can_remove && ui.button("×").clicked() {
                            return Some(ChromeAction::Remove(address.clone()));
                        }
                        if view.can_open {
                            let button = egui::Button::new(address)
                                .min_size(egui::vec2(ui.available_width(), 0.0));
                            if ui.add(button).clicked() {
                                return Some(ChromeAction::Open(address.clone()));
                            }
                        } else {
                            ui.label(address);
                        }
                        None
                    })
                    .inner;
                if hit.is_some() {
                    row_action = hit;
                }
            }
            row_action
        })
        .inner;

    ui.separator();
    let buttons = ui
        .horizontal(|ui| {
            if let Some(label) = &view.remove_all
                && ui.button(label).clicked()
            {
                return Some(ChromeAction::RemoveAll);
            }
            if let Some(label) = &view.cancel
                && ui.button(label).clicked()
            {
                return Some(ChromeAction::Cancel);
            }
            None
        })
        .inner;
    action = action.or(buttons);

    action.map(SurfaceInput::Chrome)
}

/// Render the leap-confirmation dialog: the candidate address (green if compliant,
/// red otherwise) and its action buttons.
fn confirm_ui(ui: &mut egui::Ui, view: &ConfirmView) -> Option<SurfaceInput> {
    if !view.title.is_empty() {
        ui.heading(&view.title);
    }
    if !view.instruction.is_empty() {
        ui.label(&view.instruction);
    }
    ui.separator();
    if !view.address.is_empty() {
        let color = if view.compliant {
            egui::Color32::from_rgb(0x3c, 0xb0, 0x4a)
        } else {
            egui::Color32::from_rgb(0xc0, 0x40, 0x40)
        };
        ui.colored_label(color, &view.address);
    }
    ui.separator();
    let action = ui
        .horizontal_wrapped(|ui| {
            for (label, act) in [
                (&view.confirm, ConfirmAction::Confirm),
                (&view.block, ConfirmAction::Block),
                (&view.purge, ConfirmAction::Purge),
                (&view.cancel, ConfirmAction::Cancel),
                (&view.close, ConfirmAction::Close),
            ] {
                if let Some(label) = label
                    && ui.button(label).clicked()
                {
                    return Some(act);
                }
            }
            None
        })
        .inner;
    action.map(SurfaceInput::Confirm)
}

/// Render the address-entry dialog: a text field (seeded by the engine, edited
/// live), an optional inline error, and ok/cancel. Edits report `Change`; Ok or
/// Enter reports `Submit`.
fn input_ui(ui: &mut egui::Ui, view: &mut InputView) -> Option<SurfaceInput> {
    if !view.title.is_empty() {
        ui.heading(&view.title);
    }
    if !view.instruction.is_empty() {
        ui.label(&view.instruction);
    }
    let response = ui.add(
        egui::TextEdit::singleline(&mut view.text)
            .hint_text(view.placeholder.as_str())
            .desired_width(f32::INFINITY),
    );
    let submitted = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

    if let Some(error) = &view.error {
        ui.colored_label(egui::Color32::from_rgb(0xc0, 0x40, 0x40), error);
    }

    let buttons = ui
        .horizontal(|ui| {
            if let Some(label) = &view.ok
                && ui.button(label).clicked()
            {
                return Some(InputAction::Submit(view.text.clone()));
            }
            if let Some(label) = &view.cancel
                && ui.button(label).clicked()
            {
                return Some(InputAction::Cancel);
            }
            None
        })
        .inner;

    // Precedence: an explicit Ok/Enter submit, else Cancel, else a live edit.
    let action = buttons
        .or_else(|| submitted.then(|| InputAction::Submit(view.text.clone())))
        .or_else(|| response.changed().then(|| InputAction::Change(view.text.clone())));
    action.map(SurfaceInput::Input)
}

/// Reserve a row's height at the bottom for buttons, capping a scroll area above.
fn scroll_height(ui: &egui::Ui) -> f32 {
    (ui.available_height() - 36.0).max(60.0)
}

/// Render the `language` picker: a selectable list (single-click chooses) + cancel.
fn language_ui(ui: &mut egui::Ui, view: &LanguageView) -> Option<SurfaceInput> {
    if !view.title.is_empty() {
        ui.heading(&view.title);
    }
    ui.separator();
    let chosen = egui::ScrollArea::vertical()
        .max_height(scroll_height(ui))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let mut chosen = None;
            for (id, name) in &view.languages {
                let selected = view.current.as_deref() == Some(id.as_str());
                let label = if name.is_empty() { id } else { name };
                if ui.selectable_label(selected, label).clicked() {
                    chosen = Some(id.clone());
                }
            }
            chosen
        })
        .inner;
    ui.separator();
    let cancelled = view.cancel.as_ref().is_some_and(|l| ui.button(l).clicked());

    if let Some(id) = chosen {
        Some(SurfaceInput::Language(LanguageAction::Choose(id)))
    } else if cancelled {
        Some(SurfaceInput::Language(LanguageAction::Cancel))
    } else {
        None
    }
}

/// Render the `zoom` dialog: a percent slider + default/ok/cancel.
fn zoom_ui(ui: &mut egui::Ui, view: &mut ZoomView) -> Option<SurfaceInput> {
    if !view.title.is_empty() {
        ui.heading(&view.title);
    }
    ui.separator();
    ui.add(egui::Slider::new(&mut view.percent, 50..=200).suffix(" %"));
    ui.separator();
    let action = ui
        .horizontal(|ui| {
            if let Some(label) = &view.ok
                && ui.button(label).clicked()
            {
                return Some(ZoomAction::Apply(view.percent));
            }
            if let Some(label) = &view.default_label
                && ui.button(label).clicked()
            {
                return Some(ZoomAction::Default);
            }
            if let Some(label) = &view.cancel
                && ui.button(label).clicked()
            {
                return Some(ZoomAction::Cancel);
            }
            None
        })
        .inner;
    action.map(SurfaceInput::Zoom)
}

/// Render the `update` dialog: instruction + a download button and changed-branch
/// link (both open a URI externally) + cancel.
fn update_ui(ui: &mut egui::Ui, view: &UpdateView) -> Option<SurfaceInput> {
    if !view.title.is_empty() {
        ui.heading(&view.title);
    }
    if !view.instruction.is_empty() {
        ui.label(&view.instruction);
    }
    ui.separator();
    let action = ui
        .horizontal_wrapped(|ui| {
            if let (Some(label), Some(uri)) = (&view.download, &view.update_uri)
                && ui.button(label).clicked()
            {
                return Some(UpdateAction::OpenUri(uri.clone()));
            }
            if let Some(uri) = &view.branch_uri
                && ui.link(uri).clicked()
            {
                return Some(UpdateAction::OpenUri(uri.clone()));
            }
            if let Some(label) = &view.cancel
                && ui.button(label).clicked()
            {
                return Some(UpdateAction::Cancel);
            }
            None
        })
        .inner;
    action.map(SurfaceInput::Update)
}

/// Render the `legalinformation` viewer: scrollable topic text + close.
fn legal_ui(ui: &mut egui::Ui, view: &LegalView) -> Option<SurfaceInput> {
    if !view.title.is_empty() {
        ui.heading(&view.title);
    }
    ui.separator();
    egui::ScrollArea::vertical()
        .max_height(scroll_height(ui))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (title, text) in &view.topics {
                if !title.is_empty() {
                    ui.strong(title);
                }
                if !text.is_empty() {
                    ui.label(text);
                }
                ui.separator();
            }
        });
    ui.separator();
    let closed = view.close.as_ref().is_some_and(|l| ui.button(l).clicked());
    closed.then_some(SurfaceInput::Legal(LegalAction::Close))
}

/// Render an inspector window: address + status, step/content comboboxes, the
/// content document, an autosync toggle, and synchronize/rerun/close buttons.
fn inspector_ui(ui: &mut egui::Ui, view: &mut InspectorView) -> Option<SurfaceInput> {
    let (orig_step, orig_content, orig_autosync) =
        (view.active_step, view.active_content, view.autosync_on);

    if !view.title.is_empty() {
        ui.heading(&view.title);
    }
    if !view.address.is_empty() {
        ui.label(&view.address);
    }
    if !view.status.is_empty() {
        ui.label(&view.status);
    }
    ui.separator();

    if !view.steps.is_empty() {
        let current = view
            .steps
            .get(view.active_step.max(0) as usize)
            .cloned()
            .unwrap_or_default();
        egui::ComboBox::from_label("Step")
            .selected_text(current)
            .show_ui(ui, |ui| {
                for (i, label) in view.steps.iter().enumerate() {
                    ui.selectable_value(&mut view.active_step, i as i32, label);
                }
            });
    }
    if !view.content_labels.is_empty() {
        let current = view
            .content_labels
            .get(view.active_content.max(0) as usize)
            .cloned()
            .unwrap_or_default();
        egui::ComboBox::from_label("Content")
            .selected_text(current)
            .show_ui(ui, |ui| {
                for (i, label) in view.content_labels.iter().enumerate() {
                    ui.selectable_value(&mut view.active_content, i as i32, label);
                }
            });
    }

    if !view.content.is_empty() {
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .auto_shrink([false, false])
            .show(ui, |ui| ui.monospace(&view.content));
    }

    ui.separator();
    ui.checkbox(&mut view.autosync_on, "Autosync");
    let buttons = ui
        .horizontal(|ui| {
            if view.synchronize_enabled
                && let Some(label) = &view.synchronize_label
                && ui.button(label).clicked()
            {
                return Some(InspectorAction::Synchronize);
            }
            if let Some(label) = &view.rerun_label
                && ui.button(label).clicked()
            {
                return Some(InspectorAction::Rerun);
            }
            if let Some(label) = &view.close_label
                && ui.button(label).clicked()
            {
                return Some(InspectorAction::Close);
            }
            None
        })
        .inner;

    // Button > step change > content change > autosync toggle.
    let action = buttons
        .or_else(|| {
            (view.active_step != orig_step).then_some(InspectorAction::SelectStep(view.active_step))
        })
        .or_else(|| {
            (view.active_content != orig_content)
                .then_some(InspectorAction::SelectContent(view.active_content))
        })
        .or_else(|| {
            (view.autosync_on != orig_autosync).then_some(InspectorAction::Autosync(view.autosync_on))
        });
    action.map(SurfaceInput::Inspector)
}

/// The set of slide-pixel coordinates inside a zone's silhouette — the union of
/// its [`KIND_HIT`] (2001) pieces' 1-bpp `raw` masks, each placed at its geom.
///
/// The mask is MSB-first, `ceil(w·h/8)` bytes, no row padding. A colored hover
/// piece is clipped to this set so its opaque background can't bleed past the
/// real (pill / badge / arrow) shape.
fn zone_silhouette(rollover: &Rollover) -> HashSet<(i32, i32)> {
    let mut silhouette = HashSet::new();
    for piece in &rollover.pieces {
        if piece.kind != KIND_HIT {
            continue;
        }
        let Some(mask) = &piece.raw else {
            continue;
        };
        let (w, h) = (mask.width() as i32, mask.height() as i32);
        let bytes = mask.bytes();
        for by in 0..h {
            for bx in 0..w {
                let i = (by * w + bx) as usize;
                let byte = i >> 3;
                if byte < bytes.len() && (bytes[byte] >> (7 - (i & 7))) & 1 == 1 {
                    silhouette.insert((piece.geom.x + bx, piece.geom.y + by));
                }
            }
        }
    }
    silhouette
}

/// Map a physical cursor position to slide-pixel space. The window may not be
/// exactly the slide size (DPI rounding, WM constraints), so scale by the ratio.
/// `None` if the window has no area.
fn cursor_to_slide(
    cursor: (f64, f64),
    slide: [u32; 2],
    window: (u32, u32),
) -> Option<(f64, f64)> {
    (window.0 != 0 && window.1 != 0).then(|| {
        (
            cursor.0 * slide[0] as f64 / window.0 as f64,
            cursor.1 * slide[1] as f64 / window.1 as f64,
        )
    })
}

/// Whether a slide-pixel point lies inside `rect` (top-left origin, half-open).
fn rect_contains(rect: Rect, sx: f64, sy: f64) -> bool {
    sx >= rect.x as f64
        && sx < (rect.x + rect.width) as f64
        && sy >= rect.y as f64
        && sy < (rect.y + rect.height) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: i32, y: i32, width: i32, height: i32) -> Rect {
        Rect { x, y, width, height }
    }

    #[test]
    fn cursor_maps_1to1_when_window_matches_slide() {
        // 640x480 window over a 640x480 slide: physical px == slide px.
        let p = cursor_to_slide((100.0, 50.0), [640, 480], (640, 480)).unwrap();
        assert_eq!(p, (100.0, 50.0));
    }

    #[test]
    fn cursor_scales_when_window_differs_from_slide() {
        // A 1280x960 window showing a 640x480 slide: physical px is twice slide px.
        let p = cursor_to_slide((200.0, 100.0), [640, 480], (1280, 960)).unwrap();
        assert_eq!(p, (100.0, 50.0));
    }

    #[test]
    fn zero_area_window_has_no_mapping() {
        assert!(cursor_to_slide((1.0, 1.0), [640, 480], (0, 480)).is_none());
    }

    #[test]
    fn rect_contains_is_half_open() {
        let r = rect(10, 20, 100, 30); // x:[10,110) y:[20,50)
        assert!(rect_contains(r, 10.0, 20.0)); // top-left corner included
        assert!(rect_contains(r, 109.9, 49.9));
        assert!(!rect_contains(r, 110.0, 35.0)); // right edge excluded
        assert!(!rect_contains(r, 50.0, 50.0)); // bottom edge excluded
        assert!(!rect_contains(r, 9.0, 35.0)); // left of the rect
    }

    #[test]
    fn first_matching_zone_wins() {
        let regions = [rect(0, 0, 100, 100), rect(50, 50, 100, 100)];
        // (60,60) is inside both; `position` returns the first.
        let hit = regions
            .iter()
            .position(|r| rect_contains(*r, 60.0, 60.0));
        assert_eq!(hit, Some(0));
    }
}
