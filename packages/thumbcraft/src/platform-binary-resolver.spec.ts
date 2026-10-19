import assert from 'node:assert/strict';
import { dirname, join } from 'node:path';
import { describe, it } from 'node:test';

import type { IBinaryResolver } from './binary-resolver.ts';
import { EnvBinaryResolver } from './binary-resolver.ts';
import { BinaryNotFoundError, InternalError } from './errors.ts';
import { ChainBinaryResolver, PlatformBinaryResolver } from './platform-binary-resolver.ts';
import type { ISystemInfo } from './platform.ts';

/** Raiz de node_modules usada nos testes; montada com `join` para valer em qualquer sistema. */
const MODULES = join('/app', 'node_modules');

/** Simula `require.resolve`: só conhece os pacotes instalados. */
function fakeResolve(installed: string[]) {
  return (specifier: string): string => {
    const name = specifier.replace(/\/package\.json$/, '');
    if (!installed.includes(name)) {
      throw new Error(`Cannot find module '${specifier}'`);
    }
    return join(MODULES, name, 'package.json');
  };
}

const win: ISystemInfo = { platform: 'win32', arch: 'x64', libc: null };
const linux: ISystemInfo = { platform: 'linux', arch: 'x64', libc: 'glibc' };

describe('PlatformBinaryResolver', () => {
  it('finds the Windows binary inside the installed platform package', () => {
    const resolver = new PlatformBinaryResolver(win, fakeResolve(['@thumbcraft/win32-x64-msvc']));
    assert.equal(resolver.resolve(), join(MODULES, '@thumbcraft', 'win32-x64-msvc', 'bin', 'thumbcraft.exe'));
  });

  it('finds the Linux binary inside the installed platform package', () => {
    const resolver = new PlatformBinaryResolver(linux, fakeResolve(['@thumbcraft/linux-x64-gnu']));
    assert.equal(resolver.resolve(), join(MODULES, '@thumbcraft', 'linux-x64-gnu', 'bin', 'thumbcraft'));
  });

  it('asks for the package.json of the platform package', () => {
    const seen: string[] = [];
    const resolver = new PlatformBinaryResolver(linux, (specifier) => {
      seen.push(specifier);
      return join(MODULES, '@thumbcraft', 'linux-x64-gnu', 'package.json');
    });
    resolver.resolve();
    assert.deepEqual(seen, ['@thumbcraft/linux-x64-gnu/package.json']);
  });

  it('fails clearly on an unsupported platform, naming it and the override variable', () => {
    const resolver = new PlatformBinaryResolver({ platform: 'darwin', arch: 'arm64', libc: null }, fakeResolve([]));
    assert.throws(
      () => resolver.resolve(),
      (error: unknown) => {
        assert.ok(error instanceof BinaryNotFoundError);
        assert.match(error.message, /darwin-arm64/);
        assert.match(error.message, /THUMBCRAFT_BIN/);
        return true;
      },
    );
  });

  it('names the musl case on Linux', () => {
    const resolver = new PlatformBinaryResolver({ platform: 'linux', arch: 'x64', libc: 'musl' }, fakeResolve([]));
    assert.throws(() => resolver.resolve(), /linux-x64-musl/);
  });

  it('fails clearly when the platform package is not installed', () => {
    const resolver = new PlatformBinaryResolver(win, fakeResolve([]));
    assert.throws(
      () => resolver.resolve(),
      (error: unknown) => {
        assert.ok(error instanceof BinaryNotFoundError);
        assert.match(error.message, /@thumbcraft\/win32-x64-msvc/);
        assert.match(error.message, /THUMBCRAFT_BIN/);
        return true;
      },
    );
  });

  it('puts the binary next to the package.json that was resolved', () => {
    const custom = join('/somewhere', 'else', 'package.json');
    const resolver = new PlatformBinaryResolver(linux, () => custom);
    assert.equal(resolver.resolve(), join(dirname(custom), 'bin', 'thumbcraft'));
  });
});

describe('ChainBinaryResolver', () => {
  const found = (path: string): IBinaryResolver => ({ resolve: () => path });
  const missing = (message: string): IBinaryResolver => ({
    resolve: () => {
      throw new BinaryNotFoundError(message);
    },
  });

  it('returns the first resolver that finds the binary', () => {
    assert.equal(new ChainBinaryResolver([found('/first'), found('/second')]).resolve(), '/first');
  });

  it('falls through resolvers that report BinaryNotFoundError', () => {
    assert.equal(new ChainBinaryResolver([missing('a'), found('/second')]).resolve(), '/second');
  });

  it('throws the last BinaryNotFoundError when nobody finds it', () => {
    const chain = new ChainBinaryResolver([missing('primeiro'), missing('último')]);
    assert.throws(() => chain.resolve(), /último/);
  });

  it('does not swallow unexpected errors', () => {
    const broken: IBinaryResolver = {
      resolve: () => {
        throw new InternalError('falha inesperada');
      },
    };
    assert.throws(() => new ChainBinaryResolver([broken, found('/second')]).resolve(), InternalError);
  });

  it('lets THUMBCRAFT_BIN override the platform package', () => {
    const env = new EnvBinaryResolver({ THUMBCRAFT_BIN: '/dev/thumbcraft' });
    const platform = new PlatformBinaryResolver(linux, fakeResolve(['@thumbcraft/linux-x64-gnu']));
    assert.equal(new ChainBinaryResolver([env, platform]).resolve(), '/dev/thumbcraft');
  });

  it('uses the platform package when THUMBCRAFT_BIN is not set', () => {
    const env = new EnvBinaryResolver({});
    const platform = new PlatformBinaryResolver(linux, fakeResolve(['@thumbcraft/linux-x64-gnu']));
    assert.equal(new ChainBinaryResolver([env, platform]).resolve(), join(MODULES, '@thumbcraft', 'linux-x64-gnu', 'bin', 'thumbcraft'));
  });
});
