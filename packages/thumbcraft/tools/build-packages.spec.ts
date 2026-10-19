import assert from 'node:assert/strict';
import { join } from 'node:path';
import { describe, it } from 'node:test';

import { buildAll, buildMainPackage, buildPlatformPackage, parseArgs } from './build-packages.ts';
import { FakeBuildFileSystem } from './fakes/fake-build-file-system.ts';
import { PLATFORMS } from '../src/platform.ts';

const WIN = PLATFORMS[0]!;
const LINUX = PLATFORMS[1]!;

const PLATFORMS_DIR = join('/repo', 'packages', 'platforms');
const MAIN_DIR = join('/repo', 'packages', 'thumbcraft');
const OUT = join('/repo', 'outputs', 'packages');

/** Modelo de package.json de um pacote de plataforma, como fica em packages/platforms/<id>. */
const template = (platform: (typeof PLATFORMS)[number]) =>
  JSON.stringify({ name: platform.packageName, version: '0.0.0', os: [platform.os], cpu: [platform.cpu], files: ['bin'] }, null, 2);

/** Monta o disco falso com os modelos, o pacote principal compilado e os binários. */
function scenario() {
  const fs = new FakeBuildFileSystem();
  for (const platform of PLATFORMS) {
    fs.seed(join(PLATFORMS_DIR, platform.id, 'package.json'), template(platform));
    fs.seed(join(PLATFORMS_DIR, platform.id, 'README.md'), `# ${platform.packageName}`);
  }
  fs.seed(
    join(MAIN_DIR, 'package.json'),
    JSON.stringify({
      name: 'thumbcraft',
      version: '0.1.0',
      main: './dist/index.js',
      files: ['dist'],
      scripts: { build: 'tsc', test: 'node --test' },
      devDependencies: { typescript: '^5.9.0' },
    }),
  );
  fs.seed(join(MAIN_DIR, 'README.md'), '# thumbcraft');
  fs.seed(join(MAIN_DIR, 'dist', 'index.js'), 'export {}');
  fs.seed(join(MAIN_DIR, 'dist', 'thumbcraft.js'), 'export {}');
  fs.seed(join('/bin', 'thumbcraft.exe'), Buffer.from('MZ-win'));
  fs.seed(join('/bin', 'thumbcraft'), Buffer.from('ELF-linux'));
  return fs;
}

const manifestOf = async (fs: FakeBuildFileSystem, path: string) => JSON.parse(await fs.readText(path)) as Record<string, unknown>;

describe('parseArgs', () => {
  it('reads version, binaries and output folder', () => {
    const args = parseArgs(['--version', '1.2.3', '--binary', 'win32-x64-msvc=/b/t.exe', '--binary', 'linux-x64-gnu=/b/t', '--out', '/o']);
    assert.deepEqual(args, { version: '1.2.3', binaries: { 'win32-x64-msvc': '/b/t.exe', 'linux-x64-gnu': '/b/t' }, out: '/o' });
  });

  it('leaves the output folder undefined when omitted', () => {
    assert.equal(parseArgs(['--version', '0.4.0']).out, undefined);
  });

  it('requires a plain x.y.z version', () => {
    assert.throws(() => parseArgs([]), /--version/);
    assert.throws(() => parseArgs(['--version', '1.2']), /1\.2/);
    assert.throws(() => parseArgs(['--version', '1.2.3-rc1']), /1\.2\.3-rc1/);
  });

  it('rejects an unknown platform id and a malformed --binary', () => {
    assert.throws(() => parseArgs(['--version', '1.0.0', '--binary', 'darwin-arm64=/b/t']), /darwin-arm64/);
    assert.throws(() => parseArgs(['--version', '1.0.0', '--binary', 'win32-x64-msvc']), /--binary/);
  });

  it('rejects unknown flags', () => {
    assert.throws(() => parseArgs(['--version', '1.0.0', '--nada']), /--nada/);
  });
});

describe('buildPlatformPackage', () => {
  it('writes the manifest with the requested version and keeps the template fields', async () => {
    const fs = scenario();
    await buildPlatformPackage(WIN, { version: '1.2.3', binaryPath: join('/bin', 'thumbcraft.exe'), platformsDir: PLATFORMS_DIR, outDir: OUT }, fs);
    const manifest = await manifestOf(fs, join(OUT, 'win32-x64-msvc', 'package.json'));
    assert.equal(manifest.version, '1.2.3');
    assert.equal(manifest.name, '@thumbcraft/win32-x64-msvc');
    assert.deepEqual(manifest.os, ['win32']);
  });

  it('copies the binary into bin/ under its platform name, and the README', async () => {
    const fs = scenario();
    await buildPlatformPackage(LINUX, { version: '1.2.3', binaryPath: join('/bin', 'thumbcraft'), platformsDir: PLATFORMS_DIR, outDir: OUT }, fs);
    assert.deepEqual(fs.files.get(join(OUT, 'linux-x64-gnu', 'bin', 'thumbcraft')), Buffer.from('ELF-linux'));
    assert.equal(fs.files.get(join(OUT, 'linux-x64-gnu', 'README.md')), '# @thumbcraft/linux-x64-gnu');
  });

  it('marks the binary as executable, so Linux can run it after install', async () => {
    const fs = scenario();
    await buildPlatformPackage(LINUX, { version: '1.2.3', binaryPath: join('/bin', 'thumbcraft'), platformsDir: PLATFORMS_DIR, outDir: OUT }, fs);
    assert.ok(fs.executable.has(join(OUT, 'linux-x64-gnu', 'bin', 'thumbcraft')));
  });

  it('cleans stale files from a previous build', async () => {
    const fs = scenario().seed(join(OUT, 'win32-x64-msvc', 'bin', 'velho.exe'), 'x');
    await buildPlatformPackage(WIN, { version: '1.2.3', binaryPath: join('/bin', 'thumbcraft.exe'), platformsDir: PLATFORMS_DIR, outDir: OUT }, fs);
    assert.equal(await fs.exists(join(OUT, 'win32-x64-msvc', 'bin', 'velho.exe')), false);
  });

  it('fails when the binary does not exist', async () => {
    const fs = scenario();
    await assert.rejects(
      buildPlatformPackage(WIN, { version: '1.2.3', binaryPath: join('/bin', 'nao-existe.exe'), platformsDir: PLATFORMS_DIR, outDir: OUT }, fs),
      /nao-existe/,
    );
  });

  it('fails when the template name does not match the platform', async () => {
    const fs = scenario().seed(join(PLATFORMS_DIR, 'win32-x64-msvc', 'package.json'), JSON.stringify({ name: '@thumbcraft/outro', version: '0.0.0' }));
    await assert.rejects(
      buildPlatformPackage(WIN, { version: '1.2.3', binaryPath: join('/bin', 'thumbcraft.exe'), platformsDir: PLATFORMS_DIR, outDir: OUT }, fs),
      /@thumbcraft\/win32-x64-msvc/,
    );
  });
});

describe('buildMainPackage', () => {
  it('pins the version and points optionalDependencies at the exact platform versions', async () => {
    const fs = scenario();
    await buildMainPackage({ version: '1.2.3', mainDir: MAIN_DIR, outDir: OUT }, fs);
    const manifest = await manifestOf(fs, join(OUT, 'thumbcraft', 'package.json'));
    assert.equal(manifest.version, '1.2.3');
    assert.deepEqual(manifest.optionalDependencies, {
      '@thumbcraft/win32-x64-msvc': '1.2.3',
      '@thumbcraft/linux-x64-gnu': '1.2.3',
    });
  });

  it('drops scripts and devDependencies from the published manifest, keeping the rest', async () => {
    const fs = scenario();
    await buildMainPackage({ version: '1.2.3', mainDir: MAIN_DIR, outDir: OUT }, fs);
    const manifest = await manifestOf(fs, join(OUT, 'thumbcraft', 'package.json'));
    assert.equal('scripts' in manifest, false);
    assert.equal('devDependencies' in manifest, false);
    assert.equal(manifest.main, './dist/index.js');
    assert.deepEqual(manifest.files, ['dist']);
  });

  it('copies dist and the README', async () => {
    const fs = scenario();
    await buildMainPackage({ version: '1.2.3', mainDir: MAIN_DIR, outDir: OUT }, fs);
    assert.equal(fs.files.get(join(OUT, 'thumbcraft', 'dist', 'index.js')), 'export {}');
    assert.equal(fs.files.get(join(OUT, 'thumbcraft', 'README.md')), '# thumbcraft');
  });

  it('fails, asking for a build, when dist does not exist', async () => {
    const fs = scenario();
    await fs.removeDir(join(MAIN_DIR, 'dist'));
    await assert.rejects(buildMainPackage({ version: '1.2.3', mainDir: MAIN_DIR, outDir: OUT }, fs), /npm run build/);
  });
});

describe('buildAll', () => {
  it('builds the platforms that got a binary, and the main package, returning the publish order', async () => {
    const fs = scenario();
    const built = await buildAll(
      { version: '1.2.3', binaries: { 'win32-x64-msvc': join('/bin', 'thumbcraft.exe') }, out: OUT },
      { platformsDir: PLATFORMS_DIR, mainDir: MAIN_DIR, defaultOut: '/unused' },
      fs,
    );
    assert.deepEqual(built, [join(OUT, 'win32-x64-msvc'), join(OUT, 'thumbcraft')]);
    assert.equal(await fs.exists(join(OUT, 'linux-x64-gnu')), false);
  });

  it('publishes platform packages before the main one when both binaries are given', async () => {
    const fs = scenario();
    const built = await buildAll(
      { version: '1.2.3', binaries: { 'linux-x64-gnu': join('/bin', 'thumbcraft'), 'win32-x64-msvc': join('/bin', 'thumbcraft.exe') }, out: OUT },
      { platformsDir: PLATFORMS_DIR, mainDir: MAIN_DIR, defaultOut: '/unused' },
      fs,
    );
    assert.equal(built.at(-1), join(OUT, 'thumbcraft'));
    assert.equal(built.length, 3);
  });

  it('uses the default output folder when --out is omitted', async () => {
    const fs = scenario();
    const built = await buildAll(
      { version: '1.2.3', binaries: {}, out: undefined },
      { platformsDir: PLATFORMS_DIR, mainDir: MAIN_DIR, defaultOut: OUT },
      fs,
    );
    assert.deepEqual(built, [join(OUT, 'thumbcraft')]);
  });
});
