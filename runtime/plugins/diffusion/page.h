/* Diffusion: the menu page. Off, weak, medium, and strong. The picture
   encoder reads the level (GRKIT_PLUGIN_DIFFUSION_LEVEL). */
#include "../../plugin.h"

#define DIFFUSION_LEVEL_COUNT (DIFFUSION_LEVEL_MAX+1U)
static u32 diffusion_level(volatile u32 *h) {
    u32 value=h[GRKIT_PLUGIN_DIFFUSION_LEVEL/4];
    return value<DIFFUSION_LEVEL_COUNT ? value : 0;
}
static const u16 *diffusion_title(volatile u32 *h) { return text(h,TEXT_DIFFUSION); }
static const u16 *diffusion_summary(volatile u32 *h) { return text(h,TEXT_DIFFUSION_SUMMARY+diffusion_level(h)); }
static u32 diffusion_rows(volatile u32 *h) { (void)h; return DIFFUSION_LEVEL_COUNT; }
static const u16 *diffusion_label(volatile u32 *h,u32 row) { return text(h,row ? TEXT_DIFFUSION_LEVEL+row-1 : TEXT_OFF); }
static u32 diffusion_icon(volatile u32 *h,u32 row) {
    if(h[GRKIT_HOST_MENU_LEVEL/4]==MENU_ROOT) return GRKIT_ICON_TILE(diffusion_level(h));
    return row ? GRKIT_ICON_TILE(row) : GRKIT_ICON_OFF;
}
static u32 diffusion_checked(volatile u32 *h,u32 row) { return row==diffusion_level(h); }
/* Recorded pictures only: the encoder softens the highlights of each plane. */
static void diffusion_apply(volatile u32 *h,u32 row) { h[GRKIT_PLUGIN_DIFFUSION_LEVEL/4]=row; }

/* The module tile, then the three levels. */
#define DIFFUSION_PAGE {diffusion_title,diffusion_summary,diffusion_rows,diffusion_label,diffusion_icon, \
    diffusion_checked,diffusion_apply,diffusion_level,0,0,DIFFUSION_TILES}
