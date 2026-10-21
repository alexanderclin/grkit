/* Generated from crates/grkit-core/src/plugins.rs. Do not edit: change the
   registry there, then run GRKIT_UPDATE_GENERATED=1 cargo test -p grkit-core
   --test generated. */
/* The picture.encode handlers of all plugins, in call order. */
#ifndef GRKIT_GENERATED_ENCODERS_H
#define GRKIT_GENERATED_ENCODERS_H

#include "../plugins/image-control/selective.c"
#include "../plugins/date-stamp/stamp.c"
#include "../plugins/diffusion/diffusion.c"

#define PLUGIN_ENCODERS \
    {PLUGIN_IMAGE_CONTROL,selective_wanted,selective_encode}, \
    {PLUGIN_DIFFUSION,diffusion_wanted,diffusion_encode}, \
    {PLUGIN_DATE_STAMP,stamp_wanted,stamp_encode},

#endif
