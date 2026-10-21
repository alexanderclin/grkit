/* Generated from crates/grkit-core/src/abi.rs. Do not edit: change the tables
   there, then run GRKIT_UPDATE_GENERATED=1 cargo test -p grkit-core --test
   generated. */
#ifndef GRKIT_ABI_H
#define GRKIT_ABI_H

/* The private header: the start of the registry array. ARM code addresses it
   through the literal GRKIT_HEADER_REFERENCE, which the installer relocates. */
#define GRKIT_HEADER_REFERENCE 0x58012340
#define GRKIT_HOST_HEADER_SIZE 512
#define GRKIT_HOST_MAGIC_VALUE 0x314d5247
#define GRKIT_HOST_VERSION_VALUE 1
/* Number of Image Control records. */
#define GRKIT_HOST_COUNT 0
/* Address of record 0. The records follow the header without gaps. */
#define GRKIT_HOST_RECORDS 4
/* Selected record, or 0 for native processing. */
#define GRKIT_HOST_ACTIVE 8
/* Record of the last private processing packet. A diagnostic. */
#define GRKIT_HOST_LAST_APPLIED 12
/* GRKIT_HOST_MAGIC_VALUE after the installer writes the header. */
#define GRKIT_HOST_MAGIC 16
/* GRKIT_HOST_VERSION_VALUE. */
#define GRKIT_HOST_VERSION 20
/* Context word (+0x2ac) of the last parameter build. A diagnostic. */
#define GRKIT_HOST_PROCESSING_CONTEXT 24
/* Number of parameter builds. A diagnostic. */
#define GRKIT_HOST_PROCESSING_CALLS 28
/* ADJ controller that shows the private menu, or 0. */
#define GRKIT_HOST_MENU_CONTROLLER 32
/* Private menu level: 0 native, MENU_ROOT, or MENU_MODULE (runtime/modules.h). */
#define GRKIT_HOST_MENU_LEVEL 36
/* Highlighted row of the private menu. */
#define GRKIT_HOST_MENU_CURSOR 40
/* Native ADJ slot when the private menu opened. A diagnostic. */
#define GRKIT_HOST_MENU_SLOT 44
/* Guide style previewed on the ADJ canvas while GRKIT_HOST_FRAME_PREVIEW is
   set. */
#define GRKIT_HOST_GUIDE_PREVIEW 48
/* Date stamp style: 0 Off, 1 'YY M D, 2 'YY M D H:MM, 3 M D 'YY, 4 D M 'YY. */
#define GRKIT_HOST_STAMP 52
/* Installed plugins: bit n is plugin n of grkit_core::plugins::PLUGINS and
   module n of the menu. A plugin that needs effects is set only with effects. */
#define GRKIT_HOST_PLUGINS 56
/* Nonzero while the menu rebuilds the native list. The list callback and the
   native Image Control preview ignore the changes. */
#define GRKIT_HOST_MENU_REBUILDING 60
/* Native list position, saved when the private menu opens. */
#define GRKIT_HOST_MENU_LIST 64
#define GRKIT_HOST_MENU_LIST_WORDS 2
/* The module + 1 whose native settings page the menu opened from its root row
   (Module.settings, runtime/plugin.h), or 0. While it is set, the hooks of the
   borrowed native page show the settings of that module. A change to a
   controller outside the page clears it. */
#define GRKIT_HOST_SETTINGS 72
/* Number of ADJ events. GRKIT_HOST_MENU_EVENT_RING holds the last ones. */
#define GRKIT_HOST_MENU_EVENTS 76
/* Number of Image Control selections. A diagnostic. */
#define GRKIT_HOST_MENU_APPLIES 80
/* Icon of Off, or 0 for the native icon. */
#define GRKIT_HOST_OFF_ICON 84
/* How the private menu opened: 1 or 2 across an edge of the native slots, 3 by
   touch or on the return from the native parameters. */
#define GRKIT_HOST_MENU_EDGE 88
/* MENU was pressed, and its confirm event has not arrived yet. */
#define GRKIT_HOST_BACK_PENDING 92
/* Text descriptor of the ADJ title: (length + 1) | 0x100, then the address. */
#define GRKIT_HOST_MENU_TITLE 96
#define GRKIT_HOST_MENU_TITLE_WORDS 2
/* Text descriptor of the ADJ detail line. */
#define GRKIT_HOST_MENU_DETAIL 104
#define GRKIT_HOST_MENU_DETAIL_WORDS 2
/* The last ADJ events: kind | argument << 16. */
#define GRKIT_HOST_MENU_EVENT_RING 112
#define GRKIT_HOST_MENU_EVENT_RING_WORDS 8
/* The record whose adjustments the native parameter page edits, or 0. Fn in
   GRKit Image Control stores the record with bit 0 set, and the page start
   clears bit 0. A page that native ADJ opens clears the word. */
#define GRKIT_HOST_PARAMETERS 144
/* Number of adjustment changes. The live view rebuilds its stages when it
   changes. */
#define GRKIT_HOST_PARAMETER_CHANGES 148
/* GRKIT_HOST_PARAMETER_CHANGES at the last live-view build. */
#define GRKIT_HOST_LIVE_PARAMETER_CHANGES 152
/* Icon chunk addresses: chunk n holds icon slots 40 n to 40 n + 39. The host
   icons come first in chunk 0, then the effect icons. */
#define GRKIT_HOST_ICON_CHUNKS 156
#define GRKIT_HOST_ICON_CHUNKS_WORDS 5
/* Icon of the GRKit tile. */
#define GRKIT_HOST_PLUGINS_ICON 176
/* Private toolbar cell, or 0. */
#define GRKIT_HOST_TOOLBAR_WIDGET 180
/* ADJ controller of the private toolbar cell. */
#define GRKIT_HOST_TOOLBAR_OWNER 184
/* Module that the private menu shows at MENU_MODULE. At the end of a private
   session it holds the module of the root row where the next session opens
   (menu_reopen). */
#define GRKIT_HOST_MODULE 188
/* Reserved and zero. */
#define GRKIT_HOST_RESERVED_192 192
/* Aspect id 2 variant for the photo shown in playback: 0 none (shooting uses
   the Framing ratio), 1 not 65:24, 2 65:24. */
#define GRKIT_HOST_FRAME_FILE 196
/* Record of the last live-view build: an owned record, or 0 for native
   processing. */
#define GRKIT_HOST_LIVE_RECORD 200
/* Committed composition guide style: 0 Off, 1 thirds, 2 center, 3 golden
   ratio, 4 diagonals. */
#define GRKIT_HOST_GUIDE 204
/* Framing ratio: 0 native, then the Framing rows. */
#define GRKIT_HOST_FRAME_RATIO 208
/* ADJ Framing page preview: 0 none, else the highlighted ratio + 1. */
#define GRKIT_HOST_FRAME_PREVIEW 212
/* Number of Framing paints. A diagnostic. */
#define GRKIT_HOST_FRAME_DRAWS 216
/* ADJ canvas of the preview. It stays set until a full draw erases the mask. */
#define GRKIT_HOST_FRAME_PREVIEW_CANVAS 220
/* Icon of the first plugin tile. The tiles of each plugin follow in plugin
   order (grkit_core::plugins::tiles). */
#define GRKIT_HOST_TILE_ICON 224
/* Live view controller that Framing draws on, or 0. */
#define GRKIT_HOST_FRAME_OWNER 228
/* Canvas of that controller. */
#define GRKIT_HOST_FRAME_CANVAS 232
/* Bitmap of the last Framing paint, or 0 when nothing is drawn. */
#define GRKIT_HOST_FRAME_BITMAP 236
/* Number of paints without a bitmap. A diagnostic. */
#define GRKIT_HOST_FRAME_SKIPPED 240
/* Live-view-only override: 0 none, 1 native processing, else an owned record. */
#define GRKIT_HOST_PREVIEW 244
/* Record applied by the last non-live-view parameter build, or 0. A
   diagnostic: capture metadata and the encode read the shot ring. */
#define GRKIT_HOST_STILL 248
/* Return to GRKit Image Control after the native parameter page: 1 opening, 2
   away. 3: ADJ exited from GRKit, so the next ADJ opens GRKit. 4: MENU or OK
   on GRKit passed the event that closes ADJ to the native list. */
#define GRKIT_HOST_RETURN 252
/* The words of the plugins (grkit_core::plugins::Plugin::words), in plugin
   order, then zero words. GRKIT_PLUGIN_<PLUGIN>_<WORD> gives each offset. */
#define GRKIT_HOST_PLUGIN_AREA 256
#define GRKIT_HOST_PLUGIN_AREA_WORDS 64
/* Date Imprint look: 0 classic seven-segment digits, 1 GR1s dot-matrix digits,
   2 GR II digits. */
#define GRKIT_PLUGIN_DATE_STAMP_LOOK 256
/* Grain level: 0 Off, 1 weak, 2 medium, 3 strong. The native grain stage of
   recorded pictures uses it instead of the native grain settings. */
#define GRKIT_PLUGIN_GRAIN_LEVEL 260
/* Diffusion level: 0 Off, 1 weak, 2 medium, 3 strong. The picture encoder
   screens a blurred copy of the highlights of recorded pictures back over the
   luma plane. */
#define GRKIT_PLUGIN_DIFFUSION_LEVEL 264

/* An Image Control record. The records follow the header. */
#define GRKIT_RECORD_SIZE 0x740
/* The native processing record: a prefix, then the tone curves. */
#define GRKIT_RECORD_COEFFICIENTS 0x54
/* UTF-16 effect name with its NUL. */
#define GRKIT_RECORD_NAME 0x668
/* Nonzero for a monochrome effect. */
#define GRKIT_RECORD_MONOCHROME 0x690
/* Address of the effect icon. */
#define GRKIT_RECORD_ICON 0x694
/* Effect ID with its NUL. */
#define GRKIT_RECORD_ID 0x698
/* u16 ImageTone code recorded in photos: 0x8000 | hash of the effect ID. */
#define GRKIT_RECORD_CODE 0x6f8
/* GRF extension, zero for an effect without features. */
#define GRKIT_RECORD_FEATURES 0x700
/* i16[9] signed Q13 RGB matrix after the tone curves, applied through the
   YCbCr matrix. */
#define GRKIT_RECORD_POST 0x704
/* Selective color for recorded pictures: i16 cos and sin of the target hue
   (Q12), u16 inner and outer tangents toward smaller hue angles, then toward
   larger ones (Q8), and u8 chroma gate c0 and c1. Bytes 0x724..0x72b are
   reserved and zero. */
#define GRKIT_RECORD_SELECTIVE 0x716
/* Fn adjustments of the effect: saturation, hue, high/low key, contrast,
   highlight contrast, shadow contrast, sharpness, shading, clarity. Each byte
   is (native value - 4) mod 9, so zero is neutral. A native value is 0..8, and
   4 is neutral. */
#define GRKIT_RECORD_PARAMETERS 0x72c
#define GRKIT_FEATURE_POST 1U
#define GRKIT_FEATURE_SELECTIVE 2U
/* The longest menu string, with its NUL: an Image Control summary is a prefix
   and a 16-character effect name. */
#define GRKIT_SUMMARY_UNITS 32
/* Icon slot of GR III 2.10. A target header can define a larger slot (see
   runtime/targets). */
#ifndef GRKIT_ICON_STRIDE
#define GRKIT_ICON_STRIDE 6408
#endif

/* Picture scratch after the records: the date stamp core and halo masks, then
   the shot ring. */
#define GRKIT_PICTURE_MASKS 141376
/* Shot ring: the record of each recent non-live-view parameter build, keyed by
   its shot settings pointer, which the maker-note setter (+0x18) and the JPEG
   encode configuration (+0x34) also hold. A later shot can start processing
   before an earlier one is written, so a single "last build" word is not
   enough. Word 0 counts the writes, and entry n is {key, record} at 8 + 8 n.
   The assembly computes the offset as 0x22800 + 0x40. */
#define GRKIT_SHOT_RING 141376
#define GRKIT_SHOT_ENTRIES 8
#define GRKIT_PICTURE_SCRATCH 141448

/* Menu text after the picture scratch, in each menu language: 0 English, 1
   Simplified Chinese, 2 Traditional Chinese, 3 Japanese. The installer copies
   the label table of the build to offset 0: for each language,
   GRKIT_TEXT_STRINGS u16 byte offsets from the text start, then NUL-terminated
   UTF-16 strings. At GRKIT_TEXT_SUMMARY, the installer writes the Image
   Control summary of each record in each language, GRKIT_SUMMARY_UNITS units
   each, language-major. */
#define GRKIT_TEXT_SIZE 16384
/* Everything after the records: the script array must hold it. */
#define GRKIT_TAIL_SIZE 157832
#define GRKIT_TEXT_SUMMARY 8192
/* The parameter page title of a GRKit session, after the summaries: the page,
   its vtable, and the descriptor of the effect name. */
#define GRKIT_TEXT_PARAMETER_TITLE 16368
/* The native settings page of a GRKit session, before the parameter page
   title: the borrowed submenu entry, the text descriptors, and the views that
   show settings (runtime/menu.c). */
#define GRKIT_TEXT_SETTINGS 16112
#define GRKIT_TEXT_LANGUAGES 4
#define GRKIT_TEXT_RECORDS 20
#define GRKIT_TEXT(h) ((h)[GRKIT_HOST_RECORDS/4]+(h)[GRKIT_HOST_COUNT/4]*GRKIT_RECORD_SIZE+GRKIT_PICTURE_SCRATCH)
/* String IDs of the label table, in grkit_core::menu_text order. A run of IDs
   follows the rows or values that it names. */
#define TEXT_IMAGE_CONTROL 0
#define TEXT_FRAMING 1
#define TEXT_GUIDES 2
#define TEXT_DATE_STAMP 3
#define TEXT_DATE_STYLE 4
#define TEXT_DATE_LOOK 5
#define TEXT_GRAIN 6
#define TEXT_DIFFUSION 7
#define TEXT_OFF 8
/* Framing rows 1..3: XPan 65:24, 16:9, 4:3. */
#define TEXT_RATIO 9
/* Guides rows 1..5: thirds, center, golden ratio, diagonals, thirds and
   center. */
#define TEXT_GUIDE 12
/* Date Stamp rows 1..5: the date styles. */
#define TEXT_STAMP 17
/* Date Look values 0..2: classic, GR1s, GR II. */
#define TEXT_LOOK 22
/* Grain rows 1..3: weak, medium, strong. */
#define TEXT_GRAIN_LEVEL 25
/* Diffusion rows 1..3: weak, medium, strong. */
#define TEXT_DIFFUSION_LEVEL 28
#define TEXT_IMAGE_OFF_SUMMARY 31
/* Summaries for each value, Off first. */
#define TEXT_FRAMING_SUMMARY 32
#define TEXT_GUIDES_SUMMARY 36
#define TEXT_STAMP_SUMMARY 42
#define TEXT_GRAIN_SUMMARY 48
#define TEXT_DIFFUSION_SUMMARY 52
/* The Image Control summary before an effect name, for the installer. */
#define TEXT_IMAGE_PREFIX 56
/* The About row: the GRKit version. */
#define TEXT_VERSION 57
#define GRKIT_TEXT_STRINGS 58

#endif
