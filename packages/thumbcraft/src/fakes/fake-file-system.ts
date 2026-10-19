import type { IFileSystem } from '../file-system.ts';

/** Sistema de arquivos em memória, para testar sem tocar o disco. */
export class FakeFileSystem implements IFileSystem {
  /** Conteúdo dos arquivos, por caminho. */
  readonly files = new Map<string, Buffer>();
  /** Diretórios temporários criados e ainda não removidos. */
  readonly tempDirs = new Set<string>();
  /** Todos os diretórios temporários já criados, na ordem. */
  readonly createdTempDirs: string[] = [];

  /** Lê um arquivo; lança erro se ele não existir. */
  async readFile(path: string): Promise<Buffer> {
    const data = this.files.get(path);
    if (data === undefined) {
      throw new Error(`arquivo inexistente: ${path}`);
    }
    return data;
  }

  /** Grava um arquivo em memória. */
  async writeFile(path: string, data: Buffer): Promise<void> {
    this.files.set(path, data);
  }

  /** Cria um diretório temporário de mentira, com nome previsível. */
  async makeTempDir(): Promise<string> {
    const dir = `/tmp/thumbcraft-fake-${this.createdTempDirs.length + 1}`;
    this.createdTempDirs.push(dir);
    this.tempDirs.add(dir);
    return dir;
  }

  /** Junta com barra, sempre, para os testes não dependerem da plataforma. */
  joinPath(directory: string, name: string): string {
    return `${directory}/${name}`;
  }

  /** Remove o diretório temporário e os arquivos dentro dele. */
  async removeDir(path: string): Promise<void> {
    this.tempDirs.delete(path);
    for (const file of [...this.files.keys()]) {
      if (file.startsWith(`${path}/`)) {
        this.files.delete(file);
      }
    }
  }
}
