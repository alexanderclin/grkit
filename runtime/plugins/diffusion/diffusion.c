/* Diffusion: the picture.encode handler. A digital diffusion filter for
   recorded pictures: the bright parts of the luma plane are averaged into a
   small mask in the picture scratch, blurred there, and screened back over
   the picture, so highlights glow and hard edges soften like light through
   a mist filter. Luma only: the mask is one channel, which is what fits the
   scratch (GRKIT_PICTURE_MASKS bytes) next to two upsampled glow rows.
   The levels trade threshold, strength, and radius:
   weak keeps only the strongest highlights on a short leash, strong lets
   mid-bright areas bloom wide. */
#include "../../plugin.h"

static const u8 DIFFUSION_THRESHOLD[3]={168,148,124};
static const u8 DIFFUSION_GAIN[3]={96,150,210};
static const u8 DIFFUSION_RADIUS[3]={3,5,8}; /* mask pixels at full scale */

/* One axis of a box blur over the mask, in place, with edge clamping. `temp`
   holds one line of the axis (the longer of mw, mh bytes). Three passes of a
   box read close to a Gaussian. */
static void diffusion_blur_axis(u8 *mask,u32 mw,u32 mh,u32 r,u32 vertical,u8 *temp) {
    u32 lines=vertical ? mw : mh, len=vertical ? mh : mw, i,j,pass,window=2*r+1;
    u32 reciprocal=divide(65536+window/2,window);
    for(pass=0;pass<3;++pass) {
        for(i=0;i<lines;++i) {
            u32 sum;
            for(j=0;j<len;++j) temp[j]=vertical ? mask[j*mw+i] : mask[i*mw+j];
            sum=temp[0]*(r+1);
            for(j=1;j<=r;++j) sum+=temp[j<len ? j : len-1];
            for(j=0;j<len;++j) {
                u32 v=(sum*reciprocal)>>16;
                if(vertical) mask[j*mw+i]=(u8)v; else mask[i*mw+j]=(u8)v;
                sum+=temp[j+r+1<len ? j+r+1 : len-1];
                sum-=temp[j>=r ? j-r : 0];
            }
        }
    }
}

/* Horizontally lerp one mask row into a full-width glow row. */
static void diffusion_glow_row(const u8 *mask,u32 mw,u32 mrow,u32 x0,u32 rw,u32 ds,u32 shift,u8 *out) {
    const u8 *row=mask+mrow*mw;
    u32 x;
    for(x=0;x<rw;++x) {
        u32 mx=(x0+x)>>shift, fx=((x0+x)&(ds-1))<<8;
        u32 a=row[mx<mw ? mx : mw-1], b=row[mx+1<mw ? mx+1 : mw-1];
        out[x]=(u8)(a+(((int)b-(int)a)*(int)(fx>>shift)>>8));
    }
}

static void diffusion_plane(const Plane *p,u8 *scratch,u32 level) {
    u32 x0=p->left, y0=p->top, rw=p->right-p->left, rh=p->bottom-p->top;
    u32 ds=p->width>=2048 ? 16 : p->width>=512 ? 4 : 2;
    u32 shift=ds==16 ? 4 : ds==4 ? 2 : 1;
    u32 mw=(rw+ds-1)/ds, mh=(rh+ds-1)/ds;
    u8 *mask=scratch, *temp, *glow_a, *glow_b;
    u8 lut[256];
    u32 threshold=DIFFUSION_THRESHOLD[level-1], gain=DIFFUSION_GAIN[level-1];
    u32 r=DIFFUSION_RADIUS[level-1], i, x, y, my;
    if(mw<8 || mh<8 || mw>768 || mh>512) return;
    temp=mask+mw*mh;
    glow_a=temp+(mw>mh ? mw : mh);
    glow_b=glow_a+rw;
    if((u32)(glow_b+rw-scratch)>GRKIT_PICTURE_MASKS) return;
    for(i=0;i<256;++i) lut[i]=i<=threshold ? 0 : (u8)divide((i-threshold)*255,255-threshold);
    /* Extract: average the thresholded luma of each ds x ds block. */
    for(my=0;my<mh;++my) {
        for(x=0;x<mw;++x) {
            u32 sum=0, count=0, bx=x0+x*ds, by=y0+my*ds, xx, yy;
            for(yy=by;yy<by+ds && yy<y0+rh;++yy) {
                u32 row=p->y+yy*p->pitch;
                for(xx=bx;xx<bx+ds && xx<x0+rw;++xx) { sum+=lut[BYTE(row,xx)]; ++count; }
            }
            mask[my*mw+x]=(u8)(count ? divide(sum,count) : 0);
        }
    }
    /* The radius is in full-scale mask pixels; smaller planes blur less. */
    r=(r<<4)>>shift; if(!r) r=1;
    diffusion_blur_axis(mask,mw,mh,r,0,temp);
    diffusion_blur_axis(mask,mw,mh,r,1,temp);
    /* Composite: screen the upsampled glow over the luma. */
    for(y=0;y<rh;++y) {
        u32 gy=y0+y, mrow=y>>shift, fy=((y&(ds-1))<<8)>>shift;
        u32 row=p->y+gy*p->pitch;
        diffusion_glow_row(mask,mw,mrow,x0,rw,ds,shift,glow_a);
        diffusion_glow_row(mask,mw,mrow+1<mh ? mrow+1 : mh-1,x0,rw,ds,shift,glow_b);
        for(x=0;x<rw;++x) {
            u32 g=glow_a[x]+(((int)glow_b[x]-(int)glow_a[x])*(int)fy>>8);
            u32 v=BYTE(row,x0+x);
            g=g*gain>>8; if(g>255) g=255;
            BYTE(row,x0+x)=(u8)(255-(((255-v)*(255-g))>>8));
        }
    }
}

static u32 diffusion_wanted(volatile u32 *h,const PictureSet *set) {
    (void)set;
    return h[GRKIT_PLUGIN_DIFFUSION_LEVEL/4]!=0;
}

static void diffusion_encode(volatile u32 *h,PictureSet *set) {
    u32 level=h[GRKIT_PLUGIN_DIFFUSION_LEVEL/4], i;
    if(level>3) level=3;
    if(!level) return;
    for(i=0;i<set->count;++i) diffusion_plane(&set->planes[i],set->scratch,level);
}
