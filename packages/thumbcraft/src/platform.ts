/** Informações do sistema em que o Node está rodando; vêm de uma porta para os testes não dependerem da máquina. */
export interface ISystemInfo {
  /** Valor de `process.platform` (`'win32'`, `'linux'`, `'darwin'`...). */
  platform: string;
  /** Valor de `process.arch` (`'x64'`, `'arm64'`...). */
  arch: string;
  /** Biblioteca C do Linux, ou `null` fora do Linux ou quando não foi possível descobrir. */
  libc: 'glibc' | 'musl' | null;
}

/** Uma plataforma suportada e o pacote npm que carrega o binário dela. */
export interface IPlatformSpec {
  /** Identificador curto, igual ao nome da pasta e ao sufixo do pacote (`win32-x64-msvc`). */
  id: string;
  /** Nome do pacote npm de plataforma. */
  packageName: string;
  /** Valor de `os` no `package.json` do pacote de plataforma. */
  os: 'win32' | 'linux';
  /** Valor de `cpu` no `package.json` do pacote de plataforma. */
  cpu: 'x64';
  /** Valor de `libc` no `package.json` do pacote de plataforma, ou `null` quando não se aplica. */
  libc: 'glibc' | null;
  /** Nome do executável dentro de `bin/`. */
  binaryName: string;
}

/** As duas plataformas suportadas: Windows x64 e Linux x64 com glibc. macOS, ARM e Alpine (musl) ficam de fora. */
export const PLATFORMS: readonly IPlatformSpec[] = [
  { id: 'win32-x64-msvc', packageName: '@thumbcraft/win32-x64-msvc', os: 'win32', cpu: 'x64', libc: null, binaryName: 'thumbcraft.exe' },
  { id: 'linux-x64-gnu', packageName: '@thumbcraft/linux-x64-gnu', os: 'linux', cpu: 'x64', libc: 'glibc', binaryName: 'thumbcraft' },
];

/**
 * Descobre a plataforma suportada para o sistema informado.
 * No Linux com libc desconhecida, assume glibc (o caso comum); com musl, devolve `null`.
 *
 * @param system Sistema em que o Node roda.
 * @returns A plataforma suportada, ou `null` se não houver pacote para ela.
 */
export function detectPlatform(system: ISystemInfo): IPlatformSpec | null {
  if (system.arch !== 'x64') {
    return null;
  }
  if (system.platform === 'win32') {
    return PLATFORMS.find((p) => p.os === 'win32') ?? null;
  }
  if (system.platform === 'linux' && system.libc !== 'musl') {
    return PLATFORMS.find((p) => p.os === 'linux') ?? null;
  }
  return null;
}

/** Lê o sistema real: plataforma e arquitetura do processo e, no Linux, a libc pelo relatório de diagnóstico do Node. */
export function readSystemInfo(): ISystemInfo {
  let libc: ISystemInfo['libc'] = null;
  if (process.platform === 'linux') {
    const header = (process.report?.getReport() as { header?: { glibcVersionRuntime?: string } } | undefined)?.header;
    libc = header?.glibcVersionRuntime ? 'glibc' : 'musl';
  }
  return { platform: process.platform, arch: process.arch, libc };
}
