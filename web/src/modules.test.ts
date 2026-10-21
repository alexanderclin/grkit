import { describe, expect, it } from 'vitest';
import { ALL_MODULES, planModules, type ModuleKey } from './modules';

describe('planModules', () => {
  it('omits nothing by default', () => {
    expect(planModules(ALL_MODULES, 2)).toEqual({
      includeEffects: true,
      omitted: [],
      visible: ['imageControl', 'framing', 'guides', 'dateStamp', 'grain', 'diffusion'],
    });
  });

  it('turns Image Control off by omitting it and leaving the effects out', () => {
    const enabled = new Set<ModuleKey>(['framing', 'dateStamp']);
    expect(planModules(enabled, 2)).toEqual({
      includeEffects: false,
      omitted: ['image-control', 'guides', 'grain', 'diffusion'],
      visible: ['framing', 'dateStamp'],
    });
  });

  it('shows Image Control only with effects', () => {
    expect(planModules(new Set<ModuleKey>(['imageControl']), 0).visible).toEqual([]);
    expect(planModules(new Set<ModuleKey>(['imageControl']), 1).visible).toEqual(['imageControl']);
  });

  it('omits every other plugin when only Image Control is on', () => {
    expect(planModules(new Set<ModuleKey>(['imageControl']), 1).omitted).toEqual([
      'framing',
      'guides',
      'date-stamp',
      'grain',
      'diffusion',
    ]);
  });

  it('always omits the excluded plugins, without their effects', () => {
    const offered: ModuleKey[] = ['framing', 'guides', 'dateStamp', 'grain'];
    expect(planModules(ALL_MODULES, 2, offered, ['imageControl'])).toEqual({
      includeEffects: false,
      omitted: ['image-control'],
      visible: ['framing', 'guides', 'dateStamp', 'grain'],
    });
    expect(planModules(new Set<ModuleKey>(['guides']), 2, offered, ['imageControl']).omitted).toEqual([
      'image-control',
      'framing',
      'date-stamp',
      'grain',
    ]);
  });

  it('plans only the plugins that the camera has', () => {
    const partial: ModuleKey[] = ['guides', 'dateStamp'];
    expect(planModules(ALL_MODULES, 2, partial)).toEqual({
      includeEffects: false,
      omitted: [],
      visible: ['guides', 'dateStamp'],
    });
    expect(planModules(new Set<ModuleKey>(['imageControl', 'framing', 'guides']), 2, partial)).toEqual({
      includeEffects: false,
      omitted: ['date-stamp'],
      visible: ['guides'],
    });
  });
});
