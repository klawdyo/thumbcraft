import type { IBuildFileSystem } from '../build-file-system.ts';

/** Sistema de arquivos em memória para testar o empacotamento sem tocar o disco. */
export class FakeBuildFileSystem implements IBuildFileSystem {
  /** Arquivos por caminho: texto ou bytes. */
  readonly files = new Map<string, string | Buffer>();

  /** Registra um arquivo (atalho para montar o cenário). */
  seed(path: string, content: string | Buffer): this {
    this.files.set(path, content);
    return this;
  }

  /** Lê um arquivo como texto; lança erro se não existir. */
  async readText(path: string): Promise<string> {
    const content = this.files.get(path);
    if (content === undefined) {
      throw new Error(`arquivo inexistente: ${path}`);
    }
    return content.toString();
  }

  /** Grava um arquivo de texto. */
  async writeText(path: string, text: string): Promise<void> {
    this.files.set(path, text);
  }

  /** Copia um arquivo; lança erro se a origem não existir. */
  async copyFile(from: string, to: string): Promise<void> {
    const content = this.files.get(from);
    if (content === undefined) {
      throw new Error(`arquivo inexistente: ${from}`);
    }
    this.files.set(to, content);
  }

  /** Copia todos os arquivos que estão sob a pasta de origem para a de destino. */
  async copyDir(from: string, to: string): Promise<void> {
    for (const [path, content] of [...this.files]) {
      if (path.startsWith(`${from}/`) || path.startsWith(`${from}\\`)) {
        this.files.set(to + path.slice(from.length), content);
      }
    }
  }

  /** Diz se existe um arquivo ou uma pasta (com pelo menos um arquivo) nesse caminho. */
  async exists(path: string): Promise<boolean> {
    if (this.files.has(path)) {
      return true;
    }
    return [...this.files.keys()].some((key) => key.startsWith(`${path}/`) || key.startsWith(`${path}\\`));
  }

  /** Remove uma pasta e tudo que há dentro. */
  async removeDir(path: string): Promise<void> {
    for (const key of [...this.files.keys()]) {
      if (key === path || key.startsWith(`${path}/`) || key.startsWith(`${path}\\`)) {
        this.files.delete(key);
      }
    }
  }
}
