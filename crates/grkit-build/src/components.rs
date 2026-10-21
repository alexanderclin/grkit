use crate::{
    arm::{branch, elf_text_relocations, undefined_symbols},
    firmware::FirmwareImage,
    target::HEADER_REFERENCE,
    toolchain::{run, Toolchain},
    Anchor, Patch, Reference,
};
use anyhow::{ensure, Context, Result};
use std::{collections::BTreeMap, fs, path::Path};

// All private code lives in the tail after the decoded RTOS image (GR III
// 0x540b55c0). The RTOS maps the tail read-only and executable up to
// 0x55000000, and no firmware code refers to it. BOOT does not reload it, and
// nothing clears it: after power-on, a GR III held the noise of uninitialized
// DRAM there. The installer overwrites it without a stock-byte check. The
// guards detect a firmware that extends the image into it.
pub const TAIL_BASE: u32 = 0x540b6000;
pub const TAIL_END: u32 = 0x540be000;
const RTOS_EXECUTABLE_END: u32 = 0x55000000;
pub const CONTEXTS: &[(u32, &str)] = &[
    (0x536dc124, "abfdffeb"),
    (0x536dc28c, PROCESSING_ORIGINAL),
    (0x536dc688, "943196e5903084e58b4e00eb"),
    (0x536de1a8, "0e30d4e5"),
    (0x536e8694, "9b30d4e5"),
    (0x536e87c4, "891900eb"),
    // LiveViewController's Image Control widget: load native type, map, assign.
    (
        0x531710bc,
        "e83494e5001ad3e597e204eb0040a0e10500a0e10410a0e131a608eb",
    ),
];
/// The startup literals for the `.data` copy: source, destination, and size.
/// Source plus size is the end of the RTOS image.
const GR3_GUARDS: &[(u32, &str)] = &[(0x53000450, "40b507540000005580a00300")];
// Instructions replaced by the processing hook. Its stock path in
// runtime/processing.S does the same with literal words for the addresses.
const PROCESSING_ORIGINAL: &str =
    "940d08e3f0cc08e3010545e301c545e37b9f86e201e190e74c0e08e3010545e3";
const PROCESSING_HOOKS: &[(u32, &str, bool)] = &[
    (0x536dc124, "module_parameters", true),
    (0x536dc28c, "module_processing", false),
    (0x536dc68c, "module_packet", false),
    (0x536e8694, "module_gamma_flag", false),
    (0x536e87c4, "module_gamma_curves", true),
];
pub const MENU_HOOKS: &[(u32, &str, &str)] = &[
    (0x531a48f0, "menu_native_touch_0", "f431d0e5"),
    (0x531a4904, "menu_native_touch_1", "f431d0e5"),
    (0x531a4918, "menu_native_touch_2", "f431d0e5"),
    (0x531a492c, "menu_native_touch_3", "f431d0e5"),
    (0x531a4940, "menu_native_touch_4", "f431d0e5"),
    (0x5319ff24, "menu_destroy", "0dc0a0e1"),
    (0x531a4114, "menu_refresh", "0dc0a0e1"),
    (0x531a4db8, "menu_event", "b030d1e1"),
    (0x531a3e88, "menu_exit", "7022d0e5"),
    (0x531a4cfc, "menu_changed", "0dc0a0e1"),
    (0x533966fc, "menu_text_measure", "077193e7"),
    (0x53397108, "menu_text_draw", "073193e7"),
    (0x53209fb8, "menu_icon_width", "9e3200e3"),
    (0x53209fec, "menu_icon_height", "9e3200e3"),
    (0x5320a7cc, "menu_icon_guard", "9e1200e3"),
    (0x5320a810, "menu_icon_descriptor", "033192e7"),
    (0x531a0ba0, "menu_native_adj_apply", "0dc0a0e1"),
    (0x531ab998, "menu_native_dialog_apply", "0dc0a0e1"),
    // ImageControlController (MENU page) event and exit handlers.
    (0x531ac270, "menu_native_page_event", "0dc0a0e1"),
    (0x531ab1b0, "menu_native_page_exit", "0dc0a0e1"),
];

pub const FRAMING_HOOKS: &[(u32, &str, &str, bool)] = &[
    (0x5317681c, "framing_update", "0dc0a0e1", false),
    (0x53176de4, "framing_exit", "0dc0a0e1", false),
    (0x53170b50, "framing_destroy", "0dc0a0e1", false),
    (0x53391b14, "framing_canvas", "0dc0a0e1", false),
    (0x53391be0, "framing_children", "500800eb", true),
    // Still aspect property (0xd4) handler: replaces `ldrb r1, [r4]`, the
    // requested capture aspect, so a native 3:2 request records the Framing ratio.
    (0x535423f4, "framing_capture_request_hook", "0010d4e5", true),
    // GetResolution id 2 heights (L, then each smaller size or crop step).
    (0x536a2e48, "framing_height_0", "d33ea0e3", true),
    (0x536a2e34, "framing_height_1", "a93ea0e3", true),
    (0x536a2e20, "framing_height_2", "763ea0e3", true),
    (0x536a2e08, "framing_height_3", "383400e3", true),
    // Id 2 top insets of the screennail and the thumbnail.
    (0x536a2f98, "framing_screennail_inset", "2610a0e3", true),
    (0x536a3050, "framing_thumbnail_inset", "1010a0e3", true),
    // Playback size classifier and image aspect getter.
    (0x532b39b8, "framing_classify", "0dfcffeb", true),
    (0x532b3cac, "framing_image_aspect", "50fbffea", false),
];

/// Data and single-instruction changes that let capture use the hidden GR IV
/// aspect ids 1 (4:3) and 2 (16:9) while the live view stays 3:2. GR III
/// implements only live-view modes 0 (3:2) and 3 (1:1).
pub const FRAMING_PATCHES: &[(u32, &str, &str)] = &[
    // Live-view mode table for capture ids 1, 2, 3 (read at 0x53669424): 0, 0, 3.
    (0x53d553c8, "02010300", "00000300"),
    // Second mode lookup at 0x53437170: `bhi` -> `bne`, so only id 3 reads its
    // table. Without it the live view times out after a 4:3 or 16:9 capture.
    (0x534371d8, "1500008a", "1500001a"),
    // Pipeline row 0x536a3200: return row 0 for ids 1 and 2 without the
    // ModelDefs.cpp:115 assertion, as the stock fallback does.
    (0x536a3228, "f83b0ae3", "080000ea"),
    // AF and touch image rectangle 0x535ee890: ids 1 and 2 use the 3:2
    // rectangle. Stock writes zeros for id 2 and leaves id 1 stale.
    (0x535ee8b4, "0c00000a", "0300000a"),
    (0x535ee8c0, "000052e3", "020052e1"),
];

/// Effect extension and picture processing (`runtime/picture.*`), linked into the Framing region.
pub const PICTURE_HOOKS: &[(u32, &str, &str, bool)] = &[
    // Live-view diff: `ldrb r3, [r4, #0xe]` before the gamma stage flag.
    (0x536de1a8, "module_gamma_dirty", "0e30d4e5", false),
    // YCbCrMatrix Make: the call to its calc. Applies the post-curve matrix.
    (0x536e4774, "picture_post", "e9ffffeb", true),
    // JpegEncodeMacroProcess::Execute entry: the date stamp before the encode.
    (0x535047d4, "picture_encode_entry", "0dc0a0e1", false),
    // PentaxMakernoteEntryDataSetter, tag 0x4f (ImageTone): the effect code of
    // the shot.
    (0x53608080, "picture_capture_tone", "e1fdfeeb", true),
    // PentaxMakernoteEntryDataSetter, tag 0x247 (0x536078e8): the BL to the
    // fill of the Image Control adjustments. The Fn adjustments of the shot.
    (0x536078f4, "picture_capture_parameters", "3302ffeb", true),
    // DigitalFilterProcess SetEffectParameter (0x5350304c): the Particle load,
    // then the BL to the stage gate (0x535131c0, plan +0x24).
    (0x535031b0, "grain_particle", "9537d5e5", true),
    (0x5351361c, "grain_gate", "e7feffeb", true),
];

/// The native parameter page edits the Fn adjustments of a GRKit effect
/// (`runtime/menu.c`). ImageControlDetailController: its start method, the BL
/// to its kind reader, the two BLs to the snapshot loader, and its event
/// handler. Then the Image Control model getters and setters of the Standard
/// adjustments (saturation, hue, key, contrast, highlight, shadow, sharpness,
/// shading, clarity), and the value reads of the page's list rows.
pub const PARAMETER_HOOKS: &[Hook] = &[
    (0x531b1020, "menu_parameter_start", "0dc0a0e1", false),
    (0x531b1094, "menu_parameter_kinds", "d2eeffeb", true),
    (0x531b1060, "menu_parameter_snapshot_0", "c54804eb", true),
    (0x531b10c8, "menu_parameter_snapshot_1", "ab4804eb", true),
    (0x531b0eac, "menu_parameter_event", "b030d1e1", false),
    (0x532c283c, "menu_parameter_get_0", "0dc0a0e1", false),
    (0x532c285c, "menu_parameter_get_1", "0dc0a0e1", false),
    (0x532c287c, "menu_parameter_get_2", "0dc0a0e1", false),
    (0x532c289c, "menu_parameter_get_3", "0dc0a0e1", false),
    (0x532c28bc, "menu_parameter_get_4", "0dc0a0e1", false),
    (0x532c28dc, "menu_parameter_get_5", "0dc0a0e1", false),
    (0x532c28fc, "menu_parameter_get_6", "0dc0a0e1", false),
    (0x532c2a1c, "menu_parameter_get_7", "0dc0a0e1", false),
    (0x532c2a3c, "menu_parameter_get_8", "0dc0a0e1", false),
    (0x532bd784, "menu_parameter_set_0", "0dc0a0e1", false),
    (0x532bdcd8, "menu_parameter_set_1", "0dc0a0e1", false),
    (0x532be22c, "menu_parameter_set_2", "0dc0a0e1", false),
    (0x532be8a0, "menu_parameter_set_3", "0dc0a0e1", false),
    (0x532bef14, "menu_parameter_set_4", "0dc0a0e1", false),
    (0x532bf588, "menu_parameter_set_5", "0dc0a0e1", false),
    (0x532bfbfc, "menu_parameter_set_6", "0dc0a0e1", false),
    (0x532c0da8, "menu_parameter_set_7", "0dc0a0e1", false),
    (0x532c141c, "menu_parameter_set_8", "0dc0a0e1", false),
    // The BL to the settings getter in the row text of each adjustment.
    (0x532abfc4, "menu_parameter_row_0", "d8770aeb", true),
    (0x532ac0e0, "menu_parameter_row_1", "a2770aeb", true),
    (0x532ac1fc, "menu_parameter_row_2", "6c770aeb", true),
    (0x532ac318, "menu_parameter_row_3", "36770aeb", true),
    (0x532ac434, "menu_parameter_row_4", "00770aeb", true),
    (0x532ac550, "menu_parameter_row_5", "ca760aeb", true),
    (0x532ac66c, "menu_parameter_row_6", "94760aeb", true),
    (0x532acc88, "menu_parameter_row_7", "95750aeb", true),
    (0x532acda4, "menu_parameter_row_8", "5f750aeb", true),
];

/// A GRKit root row opens a borrowed native MENU submenu as its settings page
/// (`runtime/menu.c`, `Module.settings`). The Instant Review submenu shows
/// the rows, and the Display Time and Zoom Review value pages show the values
/// of the two settings. ChangeController (its entry) ends the session at any
/// other controller. The five MOVT of the submenu kind table base load a
/// private copy of the Instant Review entry instead. Then the row count, row
/// text, apply, and current-value methods of the two value pages.
pub const SETTINGS_HOOKS: &[Hook] = &[
    (0x5316c9d8, "menu_settings_change", "0dc0a0e1", false),
    (0x531c5d20, "menu_settings_kinds_r3_0", "b33345e3", true),
    (0x531c5d88, "menu_settings_kinds_r2", "b32345e3", true),
    (0x531c60f4, "menu_settings_kinds_r8", "b38345e3", true),
    (0x531c631c, "menu_settings_kinds_r5", "b35345e3", true),
    (0x531c63f0, "menu_settings_kinds_r3_1", "b33345e3", true),
    (0x531b3c84, "menu_settings_count_0", "0600a0e3", false),
    (0x531b5b34, "menu_settings_draw_0", "0dc0a0e1", false),
    (0x531b7514, "menu_settings_apply_0", "0dc0a0e1", false),
    (0x531b8ce4, "menu_settings_current_0", "0dc0a0e1", false),
    (0x531b3c8c, "menu_settings_count_1", "0200a0e3", false),
    (0x531b5b74, "menu_settings_draw_1", "0dc0a0e1", false),
    (0x531b7544, "menu_settings_apply_1", "0dc0a0e1", false),
    (0x531b8d18, "menu_settings_current_1", "0dc0a0e1", false),
];

pub const VIEWFINDER_HOOKS: &[(u32, &str, &str, bool)] =
    &[(0x531710c4, "menu_viewfinder_icon", "97e204eb", true)];

/// Image Control tag for GRKit photos: the playback DISP page shows the effect
/// icon. Capture writes the effect code (`picture_capture_tone`).
pub const METADATA_HOOKS: &[(u32, &str, &str, bool)] = &[
    // Playback DISP page Image Control icon.
    (0x53179b7c, "menu_playback_icon", "daf404eb", true),
    // Playback parameter detail: render GRKit photos as Standard.
    (0x532b7cfc, "menu_playback_detail", "eef1ffeb", true),
];

/// In-place instruction changes: (address, before, after).
pub const INSTRUCTION_PATCHES: &[(u32, &str, &str)] = &[
    // ImageTone extraction stores 0x10000 | (tag - 0x100) instead of 0 for
    // non-native tags, so playback can match a GRKit code. Native getters still
    // accept only 1..12.
    (0x535c251c, "0020a083", "01288083"),
];

/// A hook: (address, symbol, stock instruction, BL instead of B).
pub type Hook = (u32, &'static str, &'static str, bool);
/// An in-place change: (address, stock bytes, new bytes).
pub type InstructionPatch = (u32, &'static str, &'static str);

pub use grkit_core::plugins::Point;

/// The extension point of a hook symbol. A new hook must be given a point here.
pub fn point_of(symbol: &str) -> Result<Point> {
    const TUNING: &[&str] = &[
        "menu_native_adj_apply",
        "menu_native_dialog_apply",
        "menu_native_page_event",
        "menu_native_page_exit",
        "menu_viewfinder_icon",
        "menu_playback_icon",
        "menu_playback_detail",
        "picture_post",
        "picture_capture_tone",
        "picture_capture_parameters",
    ];
    const OVERLAY: &[&str] = &[
        "framing_update",
        "framing_exit",
        "framing_destroy",
        "framing_canvas",
        "framing_children",
    ];
    Ok(
        if TUNING.contains(&symbol)
            || symbol.starts_with("module_")
            || symbol.starts_with("menu_parameter_")
        {
            Point::Tuning
        } else if OVERLAY.contains(&symbol) {
            Point::ViewfinderOverlay
        } else if symbol.starts_with("menu_") {
            Point::MenuPage
        } else if symbol.starts_with("framing_") {
            Point::CaptureAspect
        } else if symbol == "picture_encode_entry" {
            Point::PictureEncode
        } else if symbol.starts_with("grain_") {
            Point::PipelineGrain
        } else {
            anyhow::bail!("Hook {symbol} has no extension point")
        },
    )
}

/// Where a target's components live and where they hook the firmware.
#[derive(Debug, PartialEq, Eq)]
pub struct Layout {
    /// The preprocessor definition that selects `runtime/targets/<target>.h`
    /// in `runtime/firmware.h`.
    pub define: &'static str,
    /// The tail reservation after the decoded RTOS image. Each code region lies
    /// inside it. Its first 256 bytes stay unused.
    pub tail: (u32, u32),
    /// Image Control processing, if the target has it: its region and its
    /// assembly source.
    pub processing: Option<(u32, u32)>,
    pub processing_source: &'static str,
    pub menu: (u32, u32),
    /// Framing, Guides, and picture code (`runtime/framing.*`, `runtime/picture.*`).
    pub framing: Option<(u32, u32)>,
    pub contexts: &'static [(u32, &'static str)],
    pub processing_hooks: &'static [(u32, &'static str, bool)],
    /// Menu entry hooks (B), then the viewfinder and metadata hooks.
    pub menu_hooks: &'static [(u32, &'static str, &'static str)],
    pub viewfinder_hooks: &'static [Hook],
    pub metadata_hooks: &'static [Hook],
    /// The Fn parameter page hooks of the menu.
    pub parameter_hooks: &'static [Hook],
    /// The native settings page hooks of the menu, or none where the target
    /// has no settings page.
    pub settings_hooks: &'static [Hook],
    pub instruction_patches: &'static [InstructionPatch],
    pub framing_hooks: &'static [Hook],
    pub picture_hooks: &'static [Hook],
    pub framing_patches: &'static [InstructionPatch],
    /// Stock bytes that a statically patched firmware changes when it grows the
    /// RTOS image into the private tail regions. The installer checks them
    /// before it writes anything.
    pub guards: &'static [(u32, &'static str)],
    /// The installer relocates the private code to the running build of the
    /// firmware version (`crate::relocation`). The firmware addresses of the
    /// code are then `FIRMWARE()` symbols, and the tail follows the image end.
    pub relocatable: bool,
}

pub const GR3_LAYOUT: Layout = Layout {
    define: "-DGRKIT_TARGET_GR3_210",
    tail: (TAIL_BASE, TAIL_END),
    processing: Some((0x540b6100, 0x540b6500)),
    processing_source: include_str!("../../../runtime/processing.S"),
    // The menu/framing boundary sits 2 KB lower than upstream: the Diffusion
    // picture encoder needs the room in the framing component, and the menu
    // component keeps enough headroom for its pages.
    menu: (0x540b6500, 0x540b9300),
    framing: Some((0x540b9300, TAIL_END)),
    contexts: CONTEXTS,
    processing_hooks: PROCESSING_HOOKS,
    menu_hooks: MENU_HOOKS,
    viewfinder_hooks: VIEWFINDER_HOOKS,
    metadata_hooks: METADATA_HOOKS,
    parameter_hooks: PARAMETER_HOOKS,
    settings_hooks: SETTINGS_HOOKS,
    instruction_patches: INSTRUCTION_PATCHES,
    framing_hooks: FRAMING_HOOKS,
    picture_hooks: PICTURE_HOOKS,
    framing_patches: FRAMING_PATCHES,
    guards: GR3_GUARDS,
    relocatable: true,
};

/// GR IV 1.11 menu hooks.
const GR4_MENU_HOOKS: &[(u32, &str, &str)] = &[
    (0x531bf9d8, "menu_native_touch_0", "fc31d0e5"),
    (0x531bf9ec, "menu_native_touch_1", "fc31d0e5"),
    (0x531bfa00, "menu_native_touch_2", "fc31d0e5"),
    (0x531bfa14, "menu_native_touch_3", "fc31d0e5"),
    (0x531bfa28, "menu_native_touch_4", "fc31d0e5"),
    (0x531ba1c0, "menu_destroy", "0dc0a0e1"),
    (0x531bf3c8, "menu_refresh", "0dc0a0e1"),
    (0x531bf834, "menu_event", "b030d1e1"),
    (0x531bf580, "menu_exit", "fc11d0e5"),
    (0x531bf38c, "menu_changed", "0dc0a0e1"),
    (0x53572b00, "menu_text_measure", "077193e7"),
    (0x5357350c, "menu_text_draw", "073193e7"),
    // ADJ Image Control item apply (item 15, +0x28), ImageControlController
    // apply, event, and exit, as on GR III.
    (0x531badd4, "menu_native_adj_apply", "0dc0a0e1"),
    (0x531c5894, "menu_native_dialog_apply", "0dc0a0e1"),
    (0x531c635c, "menu_native_page_event", "0dc0a0e1"),
    (0x531c4fb8, "menu_native_page_exit", "0dc0a0e1"),
    // Icon lookup entry, then the limit check and the table read of the icon draw.
    (0x5323db68, "menu_icon_lookup", "0dc0a0e1"),
    (0x5323e434, "menu_icon_limit", "020057e1"),
    (0x5323e4d8, "menu_icon_entry", "073190e7"),
];

/// GR IV 1.11 Framing and Guides hooks: LiveViewController update, exit, and
/// destroy, then Canvas::Draw entry and its child-draw BL. GR IV has a
/// live-view mode for each capture aspect, so instead of the GR III patches two
/// hooks keep the live view in 3:2 while XPan uses capture aspect id 2.
const GR4_FRAMING_HOOKS: &[Hook] = &[
    (0x5318e350, "framing_update", "0dc0a0e1", false),
    (0x5318e984, "framing_exit", "0dc0a0e1", false),
    (0x531879e0, "framing_destroy", "0dc0a0e1", false),
    (0x5356dde4, "framing_canvas", "0dc0a0e1", false),
    (0x5356deb0, "framing_children", "6b0800eb", true),
    // Still aspect property (0xdf) handler, as on GR III: a native 3:2 request
    // records the Framing ratio.
    (0x5374f2ac, "framing_capture_request_hook", "0010d4e5", true),
    // The capture aspect getter BL of the live-view mode function (0x5388c7b8)
    // and of the live-view mode check (0x5363fa84).
    (0x5388c7d0, "framing_live_mode", "0d2ffbeb", true),
    (0x5363fa9c, "framing_live_check", "5a6204eb", true),
    // ImageResolutionTranslatorKb636 (the GR IV sizes) GetResolution id 2
    // heights. The Kb588 class holds the GR III sizes and is not used.
    (0x538cfff8, "framing_height_0", "983d00e3", true),
    (0x538cffe4, "framing_height_1", "ae3ea0e3", true),
    (0x538cffcc, "framing_height_2", "7b3ea0e3", true),
    (0x538cffb8, "framing_height_3", "383400e3", true),
    // Id 2 top insets of the screennail and the thumbnail.
    (0x538d00fc, "framing_screennail_inset", "2610a0e3", true),
    (0x538d01b4, "framing_thumbnail_inset", "1010a0e3", true),
    // Playback size classifier and image aspect getter.
    (0x533a2850, "framing_classify", "6dfcffeb", true),
    (0x533a2c58, "framing_image_aspect", "6bfbffea", false),
];

/// GR IV 1.11 picture hooks, as `PICTURE_HOOKS` on GR III: the live-view diff,
/// the YCbCr matrix calc call, and the JpegEncodeMacroProcess::Execute entry.
const GR4_PICTURE_HOOKS: &[Hook] = &[
    (0x5391410c, "module_gamma_dirty", "0e30d4e5", false),
    (0x5391a6a4, "picture_post", "e9ffffeb", true),
    (0x537034b4, "picture_encode_entry", "0dc0a0e1", false),
    // The ImageTone call of the maker-note setter.
    (0x53810804, "picture_capture_tone", "1a37ffeb", true),
    // The adjustments fill of tag 0x247 (handler 0x53810074).
    (0x53810080, "picture_capture_parameters", "6a3bffeb", true),
    // DigitalFilterProcess SetEffectParameter (0x53701c40): the grain on,
    // strength, and size loads, then the BL to the stage gate (0x537150fc,
    // plan +0x25).
    (0x53701e14, "grain_enable", "b537d4e5", true),
    (0x53701e34, "grain_strength", "b737d4e5", true),
    (0x53701e58, "grain_size", "b637d4e5", true),
    (0x5371557c, "grain_gate", "defeffeb", true),
];

/// GR IV 1.11 parameter page hooks, as `PARAMETER_HOOKS` on GR III. The
/// model getters and setters and the row texts of shading and clarity are
/// those of row kinds 14 and 15.
const GR4_PARAMETER_HOOKS: &[Hook] = &[
    (0x531cb6cc, "menu_parameter_start", "0dc0a0e1", false),
    (0x531cb744, "menu_parameter_kinds", "51edffeb", true),
    (0x531cb710, "menu_parameter_snapshot_0", "86cf07eb", true),
    (0x531cb77c, "menu_parameter_snapshot_1", "6bcf07eb", true),
    (0x531cb55c, "menu_parameter_event", "b030d1e1", false),
    (0x533b7110, "menu_parameter_get_0", "0dc0a0e1", false),
    (0x533b77f8, "menu_parameter_get_1", "0dc0a0e1", false),
    (0x533b7ee0, "menu_parameter_get_2", "0dc0a0e1", false),
    (0x533b8908, "menu_parameter_get_3", "0dc0a0e1", false),
    (0x533b9330, "menu_parameter_get_4", "0dc0a0e1", false),
    (0x533b9d58, "menu_parameter_get_5", "0dc0a0e1", false),
    (0x533ba780, "menu_parameter_get_6", "0dc0a0e1", false),
    (0x533bcd98, "menu_parameter_get_7", "0dc0a0e1", false),
    (0x533bd7c0, "menu_parameter_get_8", "0dc0a0e1", false),
    (0x533b670c, "menu_parameter_set_0", "0dc0a0e1", false),
    (0x533b675c, "menu_parameter_set_1", "0dc0a0e1", false),
    (0x533b67ac, "menu_parameter_set_2", "0dc0a0e1", false),
    (0x533b67fc, "menu_parameter_set_3", "0dc0a0e1", false),
    (0x533b684c, "menu_parameter_set_4", "0dc0a0e1", false),
    (0x533b689c, "menu_parameter_set_5", "0dc0a0e1", false),
    (0x533b68ec, "menu_parameter_set_6", "0dc0a0e1", false),
    (0x533b6b6c, "menu_parameter_set_7", "0dc0a0e1", false),
    (0x533b6bbc, "menu_parameter_set_8", "0dc0a0e1", false),
    // The BL to the settings getter in the row text of each adjustment.
    (0x5339a6a8, "menu_parameter_row_0", "11f50eeb", true),
    (0x5339a7c4, "menu_parameter_row_1", "dbf40eeb", true),
    (0x5339a8e0, "menu_parameter_row_2", "a5f40eeb", true),
    (0x5339a9fc, "menu_parameter_row_3", "6ff40eeb", true),
    (0x5339ab18, "menu_parameter_row_4", "39f40eeb", true),
    (0x5339ac34, "menu_parameter_row_5", "03f40eeb", true),
    (0x5339ad50, "menu_parameter_row_6", "cdf30eeb", true),
    (0x5339b450, "menu_parameter_row_7", "a6f20eeb", true),
    (0x5339b56c, "menu_parameter_row_8", "70f20eeb", true),
];

/// GR IV 1.11 settings page hooks, as `SETTINGS_HOOKS` on GR III.
const GR4_SETTINGS_HOOKS: &[Hook] = &[
    (0x53182e4c, "menu_settings_change", "0dc0a0e1", false),
    (0x531e206c, "menu_settings_kinds_r3_0", "da3345e3", true),
    (0x531e20d4, "menu_settings_kinds_r2", "da2345e3", true),
    (0x531e2444, "menu_settings_kinds_r8", "da8345e3", true),
    (0x531e266c, "menu_settings_kinds_r5", "da5345e3", true),
    (0x531e2740, "menu_settings_kinds_r3_1", "da3345e3", true),
    (0x531cf4d4, "menu_settings_count_0", "0600a0e3", false),
    (0x531d1cb0, "menu_settings_draw_0", "0dc0a0e1", false),
    (0x531d39ac, "menu_settings_apply_0", "0dc0a0e1", false),
    (0x531d5364, "menu_settings_current_0", "0dc0a0e1", false),
    (0x531cf4dc, "menu_settings_count_1", "0200a0e3", false),
    (0x531d1cf4, "menu_settings_draw_1", "0dc0a0e1", false),
    (0x531d39dc, "menu_settings_apply_1", "0dc0a0e1", false),
    (0x531d5398, "menu_settings_current_1", "0dc0a0e1", false),
];

const GR4_VIEWFINDER_HOOKS: &[Hook] = &[(0x53187f90, "menu_viewfinder_icon", "2e3d08eb", true)];

/// GR IV 1.11 Image Control tag, as `METADATA_HOOKS` on GR III: the playback
/// DISP icon and the playback parameter detail.
const GR4_METADATA_HOOKS: &[Hook] = &[
    (0x531916c0, "menu_playback_icon", "5a5408eb", true),
    (0x533a7614, "menu_playback_detail", "7af0ffeb", true),
];

/// GR IV ImageTone extraction compares each native tag value. Its default
/// case stores 0x10000 | (tag - 0x100), as on GR III, instead of 0.
const GR4_INSTRUCTION_PATCHES: &[InstructionPatch] = &[(0x537d8a54, "0020a0e3", "ff2c80e2")];

/// GR IV 1.11 Image Control processing (`runtime/targets/gr4-1.11-processing.S`).
const GR4_PROCESSING_HOOKS: &[(u32, &str, bool)] = &[
    (0x53912c70, "module_parameters", true),
    (0x53912d40, "module_processing", true),
    (0x53912d4c, "module_record", true),
    (0x53913188, "module_packet", true),
    (0x5391e360, "module_gamma_flag", true),
    (0x5391e76c, "module_gamma_curves", true),
];
/// The stock instructions around the GR IV processing hooks.
const GR4_CONTEXTS: &[(u32, &str)] = &[
    // Normalizer BL, then the type-table getters and the record store.
    (0x53912c70, "d8f9ffeb"),
    (0x53912d3c, "34a01be50a10a0e10600a0e18d4c00eb080084e5"),
    // Packet: the last field store before the native tail.
    (0x53913184, "943195e5903084e5255000eb"),
    // Gamma stage: the custom-curve flag and the curve getter BL.
    (0x5391e360, "9b10d1e5"),
    (0x5391e76c, "231a00eb"),
    // LiveViewController's Image Control widget: load native type, map, assign.
    (0x53187f88, "ec3494e5401ad3e52e3d08eb"),
];

/// The startup `.data` literals, as on GR III, and the icon catalog getter
/// (`movw r0` / `movt r0`, 0x55013410), which patched firmware moves into the
/// RTOS tail when it adds icons.
const GR4_GUARDS: &[(u32, &str)] = &[
    (0x53000450, "40b637540000005580740500"),
    (0x533ef604, "100403e3010545e3"),
];

pub const GR4_LAYOUT: Layout = Layout {
    define: "-DGRKIT_TARGET_GR4_111",
    // After the decoded RTOS image (0x543d2ac0), like GR III.
    tail: (0x543d3000, 0x543dc000),
    processing: Some((0x543d3100, 0x543d3500)),
    processing_source: include_str!("../../../runtime/targets/gr4-1.11-processing.S"),
    menu: (0x543d3500, 0x543d6800),
    framing: Some((0x543d6800, 0x543dc000)),
    contexts: GR4_CONTEXTS,
    processing_hooks: GR4_PROCESSING_HOOKS,
    menu_hooks: GR4_MENU_HOOKS,
    viewfinder_hooks: GR4_VIEWFINDER_HOOKS,
    metadata_hooks: GR4_METADATA_HOOKS,
    parameter_hooks: GR4_PARAMETER_HOOKS,
    settings_hooks: GR4_SETTINGS_HOOKS,
    instruction_patches: GR4_INSTRUCTION_PATCHES,
    framing_hooks: GR4_FRAMING_HOOKS,
    picture_hooks: GR4_PICTURE_HOOKS,
    framing_patches: &[],
    guards: GR4_GUARDS,
    relocatable: true,
};

/// Every hook of a layout: (address, symbol).
pub(crate) fn hook_sites(layout: &Layout) -> impl Iterator<Item = (u32, &'static str)> + '_ {
    layout
        .processing_hooks
        .iter()
        .map(|&(address, name, _)| (address, name))
        .chain(
            layout
                .menu_hooks
                .iter()
                .map(|&(address, name, _)| (address, name)),
        )
        .chain(
            layout
                .viewfinder_hooks
                .iter()
                .chain(layout.metadata_hooks)
                .chain(layout.parameter_hooks)
                .chain(layout.settings_hooks)
                .chain(layout.framing_hooks)
                .chain(layout.picture_hooks)
                .map(|&(address, name, _, _)| (address, name)),
        )
}

/// `hooks.h` for the assembly sources: for each hook, its address, the address
/// after it, the replaced instruction, and the target of a replaced branch.
pub fn hooks_header(body: &FirmwareImage) -> Result<String> {
    use std::fmt::Write;
    let mut result = format!("/* Generated for {}. */\n", body.target.name);
    let Some(layout) = body.target.layout else {
        return Ok(result);
    };
    // A relocatable target names each firmware address that the code uses
    // with `FIRMWARE()` (runtime/firmware.h).
    let value = |address: u32| {
        if layout.relocatable {
            format!("FIRMWARE({address:#x})")
        } else {
            format!("{address:#x}")
        }
    };
    for (address, name) in hook_sites(layout) {
        let word = u32::from_le_bytes(body.read(address, 4)?.try_into()?);
        let name = name.to_ascii_uppercase();
        writeln!(result, "#define HOOK_{name} {address:#x}")?;
        writeln!(result, "#define HOOK_{name}_RESUME {}", value(address + 4))?;
        writeln!(result, "#define HOOK_{name}_STOCK {word:#010x}")?;
        if let Some(target) = crate::relocation::branch_target(address, word) {
            writeln!(result, "#define HOOK_{name}_TARGET {}", value(target))?;
        }
    }
    Ok(result)
}

/// Stack frame limits of the private C code, which runs on the stacks of
/// firmware tasks. The largest frame is `blur_line` of `picture.c` (1344 bytes).
const MENU_FRAME_LIMIT: &str = "-Wframe-larger-than=256";
const FRAMING_FRAME_LIMIT: &str = "-Wframe-larger-than=1536";

pub struct Component {
    pub patches: Vec<Patch>,
    pub relocations: Vec<usize>,
    /// The firmware and tail references of a relocatable target. Patch 0 is
    /// the payload.
    pub references: Vec<Reference>,
    pub symbols: BTreeMap<String, u32>,
    pub code_size: usize,
    /// The size of the reserved code region.
    pub limit: usize,
}

fn code_component(
    code: Vec<u8>,
    symbols: BTreeMap<String, u32>,
    body: &FirmwareImage,
    (base, end): (u32, u32),
    references: Option<Vec<Reference>>,
) -> Result<Component> {
    ensure!(
        u64::from(base) + code.len() as u64 <= u64::from(end),
        "Private component exceeds its reserved region ({} bytes, limit {})",
        code.len(),
        end - base
    );
    // The decoded image does not contain the tail (see `TAIL_BASE`).
    let image_end = crate::target::RTOS_BASE + body.body.len() as u32;
    ensure!(
        base >= image_end && end <= RTOS_EXECUTABLE_END,
        "Private component is outside the RTOS tail"
    );
    let relocations: Vec<_> = code
        .chunks_exact(4)
        .enumerate()
        .filter_map(|(i, word)| (word == HEADER_REFERENCE.to_le_bytes()).then_some(i * 4))
        .collect();
    ensure!(!relocations.is_empty(), "Missing private header literal");
    let code_size = code.len();
    // On a relocatable target the payload moves with the tail.
    let site = references.is_some().then_some(Anchor::Tail);
    let references = references.unwrap_or_default();
    Ok(Component {
        patches: vec![Patch {
            address: base,
            points: 0,
            before: vec![0; code_size],
            after: code,
            site,
        }],
        relocations,
        references,
        symbols,
        code_size,
        limit: (end - base) as usize,
    })
}

pub fn processing(
    directory: &Path,
    runtime: &Path,
    body: &FirmwareImage,
    tools: &Toolchain,
    layout: &Layout,
    (base, end): (u32, u32),
) -> Result<Component> {
    for &(address, expected) in layout.contexts {
        ensure!(
            body.read(address, expected.len() / 2)? == hex::decode(expected)?,
            "Private processing instructions changed at {address:#x}"
        );
    }
    let source = directory.join("module-processing.S");
    let object = directory.join("module-processing.o");
    fs::write(&source, layout.processing_source)?;
    run(
        &tools.clang,
        [
            "--target=arm-none-eabi",
            "-march=armv7-a",
            "-marm",
            "-I",
            path(runtime)?,
            "-c",
            path(&source)?,
            "-o",
            path(&object)?,
        ]
        .into_iter()
        .chain([layout.define]),
    )?;
    // The firmware addresses are symbols, which the link resolves.
    let mut component = linked_component(
        directory,
        "module-processing",
        &[object],
        body,
        tools,
        (base, end),
        layout.relocatable,
    )?;
    for &(address, name, link) in layout.processing_hooks {
        let before = body.read(address, 4)?.to_vec();
        push_hook(&mut component, layout, base, (address, name, link), before)?;
    }
    Ok(component)
}

pub fn menu(
    directory: &Path,
    runtime: &Path,
    body: &FirmwareImage,
    tools: &Toolchain,
    layout: &Layout,
) -> Result<Component> {
    let (base, end) = layout.menu;
    let source = directory.join("module-menu.S");
    let object = directory.join("module-menu.o");
    let assembly_object = directory.join("module-menu-asm.o");
    fs::write(&source, include_str!("../../../runtime/menu.S"))?;
    run(
        &tools.clang,
        [
            "--target=arm-none-eabi",
            "-march=armv7-a",
            // Thumb-2 for size. menu.c keeps its firmware entry points in A32.
            "-mthumb",
            "-Oz",
            "-ffreestanding",
            "-fno-builtin",
            "-fno-stack-protector",
            "-fno-unwind-tables",
            "-fno-asynchronous-unwind-tables",
            MENU_FRAME_LIMIT,
            "-Werror=frame-larger-than",
            "-c",
            path(&runtime.join("menu.c"))?,
            "-o",
            path(&object)?,
        ]
        .into_iter()
        .chain(target_flags(layout)),
    )?;
    run(
        &tools.clang,
        [
            "--target=arm-none-eabi",
            "-march=armv7-a",
            "-marm",
            "-I",
            path(runtime)?,
            "-c",
            path(&source)?,
            "-o",
            path(&assembly_object)?,
        ]
        .into_iter()
        .chain([layout.define]),
    )?;
    let mut component = linked_component(
        directory,
        "module-menu",
        &[object, assembly_object],
        body,
        tools,
        (base, end),
        layout.relocatable,
    )?;
    let hooks = layout
        .menu_hooks
        .iter()
        .map(|&(a, n, b)| (a, n, b, false))
        .chain(layout.viewfinder_hooks.iter().copied())
        .chain(layout.metadata_hooks.iter().copied())
        .chain(layout.parameter_hooks.iter().copied())
        .chain(layout.settings_hooks.iter().copied());
    add_patches(
        &mut component,
        body,
        layout,
        base,
        (Point::Tuning, layout.instruction_patches),
        hooks,
    )?;
    Ok(component)
}

pub fn framing(
    directory: &Path,
    runtime: &Path,
    body: &FirmwareImage,
    tools: &Toolchain,
    layout: &Layout,
    (base, end): (u32, u32),
) -> Result<Component> {
    let object = directory.join("framing.o");
    let assembly_object = directory.join("framing-asm.o");
    let picture = directory.join("picture.o");
    let picture_assembly = directory.join("picture-asm.o");
    for (source, object) in [
        ("framing.c", &object),
        ("framing.S", &assembly_object),
        ("picture.c", &picture),
        ("picture.S", &picture_assembly),
    ] {
        run(
            &tools.clang,
            [
                "--target=arm-none-eabi",
                "-march=armv7-a",
                "-marm",
                "-Os",
                "-ffreestanding",
                "-fno-builtin",
                "-fno-stack-protector",
                "-fno-unwind-tables",
                "-fno-asynchronous-unwind-tables",
                FRAMING_FRAME_LIMIT,
                "-Werror=frame-larger-than",
                "-I",
                path(runtime)?,
                "-c",
                path(&runtime.join(source))?,
                "-o",
                path(object)?,
            ]
            .into_iter()
            .chain(target_flags(layout)),
        )?;
    }
    let mut component = linked_component(
        directory,
        "module-framing",
        &[object, assembly_object, picture, picture_assembly],
        body,
        tools,
        (base, end),
        layout.relocatable,
    )?;
    // The capture hook changes nothing until a Framing ratio is selected, so
    // the data patches can follow it.
    add_patches(
        &mut component,
        body,
        layout,
        base,
        (Point::CaptureAspect, layout.framing_patches),
        layout
            .framing_hooks
            .iter()
            .chain(layout.picture_hooks)
            .copied(),
    )?;
    Ok(component)
}

/// The preprocessor definition of the target, and on a relocatable target
/// no MOVW and MOVT pairs for addresses: a literal word holds each one, so
/// the installer moves it with one addition.
fn target_flags<'a>(layout: &Layout) -> impl Iterator<Item = &'a str> {
    let define: &'a str = layout.define;
    std::iter::once(define).chain(layout.relocatable.then_some("-mno-movt"))
}

/// The prefix of the symbol that `FIRMWARE(address)` names (runtime/firmware.h).
const FIRMWARE_SYMBOL: &str = "fw_0x";

fn linked_component(
    directory: &Path,
    name: &str,
    objects: &[std::path::PathBuf],
    body: &FirmwareImage,
    tools: &Toolchain,
    (base, end): (u32, u32),
    relocatable: bool,
) -> Result<Component> {
    use std::fmt::Write;
    let mut script = format!("SECTIONS {{ . = {base:#x}; .text : {{ *(.text*) *(.rodata*) }} /DISCARD/ : {{ *(.ARM.exidx*) *(.comment*) }} }}\n");
    // Each firmware symbol has its address in the pinned image. Other
    // undefined symbols are those of another object, or LLD reports them.
    let mut firmware = std::collections::BTreeSet::new();
    for object in objects {
        for symbol in undefined_symbols(&fs::read(object)?)? {
            if let Some(hex) = symbol.strip_prefix(FIRMWARE_SYMBOL) {
                let address = u32::from_str_radix(hex, 16)
                    .with_context(|| format!("Invalid firmware symbol {symbol}"))?;
                firmware.insert(address);
            }
        }
    }
    for address in firmware {
        writeln!(script, "{FIRMWARE_SYMBOL}{address:x} = {address:#x};")?;
    }
    let script_path = directory.join(format!("{name}.ld"));
    fs::write(&script_path, script)?;
    let elf = directory.join(format!("{name}.elf"));
    // A branch between ARM and Thumb code that cannot interwork goes through
    // a thunk that LLD adds. A position-independent thunk moves with the
    // private code. An absolute one holds its target address in MOVW and MOVT
    // without a relocation record, so the installer cannot move it.
    let mut arguments = vec![
        "--pic-veneer".to_owned(),
        "-T".to_owned(),
        path(&script_path)?.to_owned(),
        "-o".to_owned(),
        path(&elf)?.to_owned(),
    ];
    if relocatable {
        arguments.push("--emit-relocs".to_owned());
    }
    for object in objects {
        arguments.push(path(object)?.to_owned());
    }
    run(&tools.lld, arguments.iter().map(String::as_str))?;
    let (code, absolute, relocations) = elf_text_relocations(&fs::read(elf)?)?;
    ensure!(
        relocatable || relocations.is_empty(),
        "Unresolved text relocations"
    );
    if let Some(thunk) = absolute
        .keys()
        .find(|symbol| symbol.contains("ABSLongThunk"))
    {
        anyhow::bail!("{name} has the absolute thunk {thunk}, which the installer cannot move");
    }
    let symbols = absolute
        .into_iter()
        .map(|(symbol, value)| {
            Ok((
                symbol,
                value
                    .checked_sub(base)
                    .context("Component symbol outside code")?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    let mut references = Vec::new();
    for relocation in relocations {
        // R_ARM_ABS32 and R_ARM_TARGET1 hold an address. Other types against
        // the private code are PC-relative, so they move with it.
        let word = matches!(relocation.kind, 2 | 38);
        let anchor = if relocation.internal {
            if !word {
                ensure!(
                    !matches!(relocation.kind, 5 | 6 | 8 | 43 | 44 | 47 | 48),
                    "Unsupported relocation type {} of {} in {name}",
                    relocation.kind,
                    relocation.symbol
                );
                continue;
            }
            Anchor::Tail
        } else {
            let address = relocation
                .symbol
                .strip_prefix(FIRMWARE_SYMBOL)
                .and_then(|hex| u32::from_str_radix(hex, 16).ok())
                .filter(|address| *address == relocation.value)
                .with_context(|| format!("Unexpected symbol {} in {name}", relocation.symbol))?;
            // R_ARM_CALL and R_ARM_JUMP24: an ARM B or BL to the firmware.
            ensure!(
                word || matches!(relocation.kind, 28 | 29),
                "Unsupported relocation type {} of {} in {name}",
                relocation.kind,
                relocation.symbol
            );
            Anchor::Firmware(address)
        };
        ensure!(relocation.offset % 4 == 0, "Unaligned relocation in {name}");
        references.push(Reference {
            patch: 0,
            offset: relocation.offset as usize,
            branch: !word,
            anchor,
        });
    }
    code_component(
        code,
        symbols,
        body,
        (base, end),
        relocatable.then_some(references),
    )
}

/// Append the entry branch of a hook into the component code. On a
/// relocatable target the branch moves with its site and with the tail.
fn push_hook(
    component: &mut Component,
    layout: &Layout,
    base: u32,
    (address, name, link): (u32, &str, bool),
    before: Vec<u8>,
) -> Result<()> {
    let target = base
        .checked_add(
            *component
                .symbols
                .get(name)
                .with_context(|| format!("Missing symbol {name}"))?,
        )
        .context("Symbol overflow")?;
    let mut instruction = u32::from_le_bytes(branch(address, target, 14)?);
    if link {
        instruction |= 0x01000000;
    }
    let site = layout.relocatable.then_some(Anchor::Firmware(address));
    if site.is_some() {
        component.references.push(Reference {
            patch: component.patches.len(),
            offset: 0,
            branch: true,
            anchor: Anchor::Tail,
        });
    }
    component.patches.push(Patch {
        address,
        points: point_of(name)?.bit(),
        before,
        after: instruction.to_le_bytes().to_vec(),
        site,
    });
    Ok(())
}

/// Append entry branches into the component code, then in-place changes.
fn add_patches<'a>(
    component: &mut Component,
    body: &FirmwareImage,
    layout: &Layout,
    base: u32,
    instructions: (Point, &[(u32, &str, &str)]),
    hooks: impl Iterator<Item = (u32, &'a str, &'a str, bool)>,
) -> Result<()> {
    for (address, name, expected, link) in hooks {
        let before = hex::decode(expected)?;
        ensure!(
            body.read(address, 4)? == before,
            "Private hook instruction differs at {address:#x}"
        );
        push_hook(component, layout, base, (address, name, link), before)?;
    }
    let (point, instructions) = instructions;
    for &(address, before, after) in instructions {
        let before = hex::decode(before)?;
        ensure!(
            body.read(address, before.len())? == before,
            "Private instruction differs at {address:#x}"
        );
        component.patches.push(Patch {
            address,
            points: point.bit(),
            before,
            after: hex::decode(after)?,
            site: layout.relocatable.then_some(Anchor::Firmware(address)),
        });
    }
    Ok(())
}

pub(crate) fn path(path: &Path) -> Result<&str> {
    path.to_str().context("Toolchain path is not UTF-8")
}
