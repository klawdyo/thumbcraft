import { chmod, cp, mkdir, readFile, rm, stat, writeFile } from 'node:fs/promises';
import { dirname } from 'node:path';

/** Porta para o disco usada pelo empacotamento. Os testes usam uma versão em memória. */
export interface IBuildFileSystem {
  /** Lê um arquivo como texto UTF-8. */
  readText(path: string): Promise<string>;
  /** Grava um arquivo de texto, criando as pastas que faltarem. */
  writeText(path: string, text: string): Promise<void>;
  /** Copia um arquivo (texto ou binário), criando as pastas de destino. */
  copyFile(from: string, to: string): Promise<void>;
  /** Marca um arquivo como executável (modo 0755); no Windows o sistema de arquivos não guarda o bit. */
  makeExecutable(path: string): Promise<void>;
  /** Copia uma pasta inteira, recursivamente. */
  copyDir(from: string, to: string): Promise<void>;
  /** Diz se o caminho existe. */
  exists(path: string): Promise<boolean>;
  /** Remove uma pasta e tudo que há dentro; não falha se ela não existir. */
  removeDir(path: string): Promise<void>;
}

/** Implementação real, sobre `node:fs/promises`. */
export class NodeBuildFileSystem implements IBuildFileSystem {
  /** Lê um arquivo como texto UTF-8. */
  readText(path: string): Promise<string> {
    return readFile(path, 'utf8');
  }

  /** Grava um arquivo de texto, criando as pastas que faltarem. */
  async writeText(path: string, text: string): Promise<void> {
    await mkdir(dirname(path), { recursive: true });
    await writeFile(path, text, 'utf8');
  }

  /** Copia um arquivo, criando as pastas de destino. */
  async copyFile(from: string, to: string): Promise<void> {
    await mkdir(dirname(to), { recursive: true });
    await cp(from, to);
  }

  /** Marca um arquivo como executável (modo 0755). */
  async makeExecutable(path: string): Promise<void> {
    await chmod(path, 0o755);
  }

  /** Copia uma pasta inteira, recursivamente. */
  async copyDir(from: string, to: string): Promise<void> {
    await cp(from, to, { recursive: true });
  }

  /** Diz se o caminho existe. */
  async exists(path: string): Promise<boolean> {
    try {
      await stat(path);
      return true;
    } catch {
      return false;
    }
  }

  /** Remove uma pasta e tudo que há dentro. */
  async removeDir(path: string): Promise<void> {
    await rm(path, { recursive: true, force: true });
  }
}
