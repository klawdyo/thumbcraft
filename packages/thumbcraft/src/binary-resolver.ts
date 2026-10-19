import { BinaryNotFoundError } from './errors.ts';

/** Porta que descobre onde está o binário `thumbcraft`. A tarefa dos pacotes de plataforma fornece outra implementação. */
export interface IBinaryResolver {
  /**
   * Devolve o caminho do binário.
   * @throws {BinaryNotFoundError} Quando não há como encontrá-lo.
   */
  resolve(): string;
}

/** Resolve o binário pela variável de ambiente `THUMBCRAFT_BIN`, útil no desenvolvimento. */
export class EnvBinaryResolver implements IBinaryResolver {
  private readonly env: Record<string, string | undefined>;

  /** @param env Variáveis de ambiente a consultar; por padrão, as do processo. */
  constructor(env: Record<string, string | undefined> = process.env) {
    this.env = env;
  }

  /** Devolve o valor de `THUMBCRAFT_BIN` ou lança erro com a dica de como definir. */
  resolve(): string {
    const path = this.env.THUMBCRAFT_BIN;
    if (path === undefined || path === '') {
      throw new BinaryNotFoundError(
        'binário do thumbcraft não encontrado: defina a variável de ambiente THUMBCRAFT_BIN com o caminho do executável',
      );
    }
    return path;
  }
}
