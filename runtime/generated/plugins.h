/* Generated from crates/grkit-core/src/plugins.rs. Do not edit: change the
   registry there, then run GRKIT_UPDATE_GENERATED=1 cargo test -p grkit-core
   --test generated. */
/* The plugins in menu order. */
#ifndef GRKIT_GENERATED_PLUGINS_H
#define GRKIT_GENERATED_PLUGINS_H

/* Plugin n is bit n of a selection (GRKIT_HOST_PLUGINS) and module n of the
   menu. */
#define PLUGIN_IMAGE_CONTROL 0U
#define PLUGIN_FRAMING 1U
#define PLUGIN_GUIDES 2U
#define PLUGIN_DATE_STAMP 3U
#define PLUGIN_GRAIN 4U
#define PLUGIN_DIFFUSION 5U
#define PLUGIN_COUNT 6U
/* The PLUGINS.TXT names in plugin order. */
#define PLUGIN_NAMES "image-control", "framing", "guides", "date-stamp", "grain", "diffusion"
/* The plugins that show only with effects. Without effects, the installer and
   the menu treat them as not installed. */
#define PLUGIN_NEEDS_EFFECTS 0x1U
/* The artwork tiles of each plugin. They follow each other in plugin order and
   end below the GRKit tile. */
#define IMAGE_CONTROL_TILES 0U
#define FRAMING_TILES 4U
#define GUIDES_TILES 6U
#define DATE_STAMP_TILES 6U
#define GRAIN_TILES 4U
#define DIFFUSION_TILES 4U
#define PLUGIN_TILES 24U
/* The largest value of each state key. The loader restores a larger value as
   0. */
#define IMAGE_CONTROL_EFFECT_MAX 65535U
#define FRAMING_RATIO_MAX 3U
#define GUIDES_STYLE_MAX 5U
#define DATE_STAMP_STYLE_MAX 5U
#define DATE_STAMP_LOOK_MAX 2U
#define GRAIN_LEVEL_MAX 3U
#define DIFFUSION_LEVEL_MAX 3U

#endif
