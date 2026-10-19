import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import { PLATFORMS, detectPlatform } from './platform.ts';

describe('detectPlatform', () => {
  it('recognizes Windows x64', () => {
    const platform = detectPlatform({ platform: 'win32', arch: 'x64', libc: null });
    assert.equal(platform?.id, 'win32-x64-msvc');
    assert.equal(platform?.packageName, '@thumbcraft/win32-x64-msvc');
    assert.equal(platform?.binaryName, 'thumbcraft.exe');
  });

  it('recognizes Linux x64 with glibc', () => {
    const platform = detectPlatform({ platform: 'linux', arch: 'x64', libc: 'glibc' });
    assert.equal(platform?.id, 'linux-x64-gnu');
    assert.equal(platform?.packageName, '@thumbcraft/linux-x64-gnu');
    assert.equal(platform?.binaryName, 'thumbcraft');
  });

  it('assumes glibc on Linux x64 when the libc is unknown', () => {
    assert.equal(detectPlatform({ platform: 'linux', arch: 'x64', libc: null })?.id, 'linux-x64-gnu');
  });

  it('rejects Linux with musl (Alpine)', () => {
    assert.equal(detectPlatform({ platform: 'linux', arch: 'x64', libc: 'musl' }), null);
  });

  it('rejects every other combination', () => {
    for (const [platform, arch] of [
      ['linux', 'arm64'],
      ['darwin', 'arm64'],
      ['darwin', 'x64'],
      ['win32', 'arm64'],
      ['win32', 'ia32'],
      ['freebsd', 'x64'],
    ] as const) {
      assert.equal(detectPlatform({ platform, arch, libc: null }), null, `${platform}-${arch}`);
    }
  });
});

describe('PLATFORMS', () => {
  it('lists exactly the two supported platforms with matching npm fields', () => {
    assert.deepEqual(
      PLATFORMS.map((p) => [p.id, p.os, p.cpu, p.libc]),
      [
        ['win32-x64-msvc', 'win32', 'x64', null],
        ['linux-x64-gnu', 'linux', 'x64', 'glibc'],
      ],
    );
  });

  it('keeps package names unique and scoped', () => {
    const names = PLATFORMS.map((p) => p.packageName);
    assert.equal(new Set(names).size, names.length);
    for (const name of names) {
      assert.match(name, /^@thumbcraft\//);
    }
  });
});
