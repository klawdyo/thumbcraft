import { createRequire } from 'node:module';
import { dirname, join } from 'node:path';

import { EnvBinaryResolver, type IBinaryResolver } from './binary-resolver.ts';
import { BinaryNotFoundError } from './errors.ts';
import { detectPlatform, type ISystemInfo, readSystemInfo } from './platform.ts';

/** Equivale a `require.resolve`: devolve o caminho de um arquivo de um pacote instalado ou lança erro. */
export type ResolvePackageFile = (specifier: string) => string;

/** Resolve o binário dentro do pacote de plataforma instalado (`@thumbcraft/<plataforma>`), como o `sharp` faz. */
export class PlatformBinaryResolver implements IBinaryResolver {
  private readonly system: ISystemInfo;
  private readonly resolvePackageFile: ResolvePackageFile;

  /**
   * @param system Sistema em que o Node roda.
   * @param resolvePackageFile Resolução de arquivo de pacote; por padrão, o `require.resolve` deste módulo.
   */
  constructor(system: ISystemInfo = readSystemInfo(), resolvePackageFile: ResolvePackageFile = createRequire(import.meta.url).resolve) {
    this.system = system;
    this.resolvePackageFile = resolvePackageFile;
  }

  /**
   * Devolve o caminho do executável do pacote de plataforma.
   * @throws {BinaryNotFoundError} Se a plataforma não é suportada ou o pacote dela não está instalado.
   */
  resolve(): string {
    const platform = detectPlatform(this.system);
    if (platform === null) {
      const libc = this.system.libc === 'musl' ? '-musl' : '';
      throw new BinaryNotFoundError(
        `plataforma não suportada: ${this.system.platform}-${this.system.arch}${libc}; ` +
          'há binários só para Windows x64 e Linux x64 (glibc); defina THUMBCRAFT_BIN com o caminho de um binário compilado',
      );
    }
    let packageJson: string;
    try {
      packageJson = this.resolvePackageFile(`${platform.packageName}/package.json`);
    } catch {
      throw new BinaryNotFoundError(
        `o pacote ${platform.packageName} não está instalado; reinstale o thumbcraft sem --omit=optional ` +
          'ou defina THUMBCRAFT_BIN com o caminho do binário',
      );
    }
    return join(dirname(packageJson), 'bin', platform.binaryName);
  }
}

/** Tenta cada resolvedor em ordem e devolve o primeiro caminho encontrado. */
export class ChainBinaryResolver implements IBinaryResolver {
  private readonly resolvers: readonly IBinaryResolver[];

  /** @param resolvers Resolvedores, do mais prioritário para o menos. */
  constructor(resolvers: readonly IBinaryResolver[]) {
    this.resolvers = resolvers;
  }

  /**
   * Devolve o caminho do primeiro resolvedor que acha o binário.
   * Só `BinaryNotFoundError` faz passar ao próximo; qualquer outro erro é propagado.
   * @throws {BinaryNotFoundError} O erro do último resolvedor, quando nenhum acha.
   */
  resolve(): string {
    let last: BinaryNotFoundError | undefined;
    for (const resolver of this.resolvers) {
      try {
        return resolver.resolve();
      } catch (error) {
        if (!(error instanceof BinaryNotFoundError)) {
          throw error;
        }
        last = error;
      }
    }
    throw last ?? new BinaryNotFoundError('nenhum resolvedor de binário configurado');
  }
}

/** Resolvedor padrão: `THUMBCRAFT_BIN` (para desenvolvimento) e, se ausente, o pacote de plataforma instalado. */
export function createDefaultBinaryResolver(): IBinaryResolver {
  return new ChainBinaryResolver([new EnvBinaryResolver(), new PlatformBinaryResolver()]);
}
