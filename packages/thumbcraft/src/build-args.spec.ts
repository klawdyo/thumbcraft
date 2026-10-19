import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import { buildArgs } from './render-options.ts';

describe('buildArgs', () => {
  it('without options produces no flags', () => {
    assert.deepEqual(buildArgs(), []);
    assert.deepEqual(buildArgs({}), []);
  });

  it('maps size, fit, gravity and background', () => {
    assert.deepEqual(
      buildArgs({ width: 320, height: 200, fit: 'contain', gravity: 'top-left', background: '#FF8000' }),
      ['-W', '320', '-H', '200', '--fit', 'contain', '--gravity', 'top-left', '--background', '#FF8000'],
    );
  });

  it('maps trim and trimPad in native pixels', () => {
    assert.deepEqual(buildArgs({ trim: true, trimPad: 4 }), ['--trim', '--trim-pad', '4']);
    assert.deepEqual(buildArgs({ trim: false }), []);
    assert.deepEqual(buildArgs({ trimPad: 0 }), ['--trim-pad', '0']);
  });

  it('maps page, scale and filter', () => {
    assert.deepEqual(buildArgs({ page: 3, scale: 2.5, filter: 'area' }), ['--page', '3', '--render-scale', '2.5', '--filter', 'area']);
    assert.deepEqual(buildArgs({ scale: 'auto' }), ['--render-scale', 'auto']);
  });

  it('maps crop as fractions x,y,width,height', () => {
    assert.deepEqual(buildArgs({ crop: { x: 0.25, y: 0, width: 0.5, height: 1 } }), ['--crop', '0.25,0,0.5,1']);
  });

  it('maps format and quality', () => {
    assert.deepEqual(buildArgs({ format: 'webp', quality: 70 }), ['--format', 'webp', '--quality', '70']);
  });

  it('keeps zero values instead of dropping them', () => {
    assert.deepEqual(buildArgs({ page: 0 }), ['--page', '0']);
  });
});
