import { join } from 'node:path';

import { PLATFORMS, type IPlatformSpec } from '../src/platform.ts';
import type { IBuildFileSystem } from './build-file-system.ts';

/** Argumentos da linha de comando do empacotamento. */
export interface IBuildArgs {
  /** Versão `x.y.z` de todos os pacotes (a do `Cargo.toml`). */
  version: string;
  /** Caminho do binário compilado, por identificador de plataforma; só as plataformas informadas são montadas. */
  binaries: Record<string, string>;
  /** Pasta de saída; `undefined` usa a padrão (`outputs/packages`). */
  out: string | undefined;
}

/** Pastas do repositório usadas pelo empacotamento. */
export interface IBuildPaths {
  /** Pasta com os modelos dos pacotes de plataforma (`packages/platforms`). */
  platformsDir: string;
  /** Pasta do pacote principal (`packages/thumbcraft`). */
  mainDir: string;
  /** Pasta de saída padrão. */
  defaultOut: string;
}

/** Converte os argumentos da linha de comando, validando versão, plataformas e opções. */
export function parseArgs(argv: readonly string[]): IBuildArgs {
  let version: string | undefined;
  let out: string | undefined;
  const binaries: Record<string, string> = {};
  for (let i = 0; i < argv.length; i += 1) {
    const flag = argv[i] as string;
    if (flag === '--version') {
      version = argv[++i];
    } else if (flag === '--out') {
      out = argv[++i];
    } else if (flag === '--binary') {
      const value = argv[++i] ?? '';
      const separator = value.indexOf('=');
      if (separator <= 0 || separator === value.length - 1) {
        throw new Error(`--binary inválido: '${value}' (use <plataforma>=<caminho do binário>)`);
      }
      const id = value.slice(0, separator);
      if (!PLATFORMS.some((platform) => platform.id === id)) {
        throw new Error(`plataforma desconhecida em --binary: ${id} (use ${PLATFORMS.map((p) => p.id).join(' ou ')})`);
      }
      binaries[id] = value.slice(separator + 1);
    } else {
      throw new Error(`opção desconhecida: ${flag}`);
    }
  }
  if (version === undefined) {
    throw new Error('--version é obrigatório (use x.y.z, a versão do Cargo.toml)');
  }
  if (!/^\d+\.\d+\.\d+$/.test(version)) {
    throw new Error(`versão inválida: ${version} (use x.y.z, só números)`);
  }
  return { version, binaries, out };
}

/** Lê e interpreta um package.json. */
async function readManifest(path: string, fs: IBuildFileSystem): Promise<Record<string, unknown>> {
  return JSON.parse(await fs.readText(path)) as Record<string, unknown>;
}

/** Serializa um package.json com indentação de 2 espaços e quebra de linha final. */
const serialize = (manifest: Record<string, unknown>): string => `${JSON.stringify(manifest, null, 2)}\n`;

/** Dados para montar o pacote de uma plataforma. */
export interface IPlatformBuild {
  /** Versão `x.y.z`. */
  version: string;
  /** Caminho do binário compilado dessa plataforma. */
  binaryPath: string;
  /** Pasta com os modelos de pacote de plataforma. */
  platformsDir: string;
  /** Pasta de saída. */
  outDir: string;
}

/**
 * Monta o pacote de uma plataforma em `<outDir>/<id>`: manifesto do modelo com a versão pedida, README e o binário em `bin/`.
 * @returns A pasta do pacote montado.
 */
export async function buildPlatformPackage(platform: IPlatformSpec, build: IPlatformBuild, fs: IBuildFileSystem): Promise<string> {
  const templateDir = join(build.platformsDir, platform.id);
  const manifest = await readManifest(join(templateDir, 'package.json'), fs);
  if (manifest.name !== platform.packageName) {
    throw new Error(`o modelo de ${platform.id} tem o nome '${String(manifest.name)}', esperado ${platform.packageName}`);
  }
  if (!(await fs.exists(build.binaryPath))) {
    throw new Error(`binário não encontrado para ${platform.id}: ${build.binaryPath}`);
  }
  const destination = join(build.outDir, platform.id);
  await fs.removeDir(destination);
  await fs.writeText(join(destination, 'package.json'), serialize({ ...manifest, version: build.version }));
  await fs.copyFile(join(templateDir, 'README.md'), join(destination, 'README.md'));
  await fs.copyFile(build.binaryPath, join(destination, 'bin', platform.binaryName));
  return destination;
}

/** Dados para montar o pacote principal. */
export interface IMainBuild {
  /** Versão `x.y.z`. */
  version: string;
  /** Pasta do pacote principal, com `dist/` já compilado. */
  mainDir: string;
  /** Pasta de saída. */
  outDir: string;
}

/**
 * Monta o pacote principal em `<outDir>/thumbcraft`: `dist/`, README e um manifesto sem scripts nem devDependencies,
 * com a versão pedida e `optionalDependencies` apontando para a versão exata de cada pacote de plataforma.
 * @returns A pasta do pacote montado.
 */
export async function buildMainPackage(build: IMainBuild, fs: IBuildFileSystem): Promise<string> {
  const dist = join(build.mainDir, 'dist');
  if (!(await fs.exists(dist))) {
    throw new Error(`dist não encontrado em ${dist}: rode npm run build antes de empacotar`);
  }
  const { scripts: _scripts, devDependencies: _devDependencies, ...manifest } = await readManifest(join(build.mainDir, 'package.json'), fs);
  const optionalDependencies = Object.fromEntries(PLATFORMS.map((platform) => [platform.packageName, build.version]));
  const destination = join(build.outDir, 'thumbcraft');
  await fs.removeDir(destination);
  await fs.writeText(join(destination, 'package.json'), serialize({ ...manifest, version: build.version, optionalDependencies }));
  await fs.copyDir(dist, join(destination, 'dist'));
  await fs.copyFile(join(build.mainDir, 'README.md'), join(destination, 'README.md'));
  return destination;
}

/**
 * Monta os pacotes de plataforma que receberam um binário e o pacote principal.
 * @returns As pastas montadas, na ordem de publicação: plataformas primeiro, principal por último.
 */
export async function buildAll(args: IBuildArgs, paths: IBuildPaths, fs: IBuildFileSystem): Promise<string[]> {
  const outDir = args.out ?? paths.defaultOut;
  const built: string[] = [];
  for (const platform of PLATFORMS) {
    const binaryPath = args.binaries[platform.id];
    if (binaryPath !== undefined) {
      built.push(await buildPlatformPackage(platform, { version: args.version, binaryPath, platformsDir: paths.platformsDir, outDir }, fs));
    }
  }
  built.push(await buildMainPackage({ version: args.version, mainDir: paths.mainDir, outDir }, fs));
  return built;
}
