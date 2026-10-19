import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

/** Porta para o disco, só com o que o wrapper precisa. Os testes usam uma versão em memória. */
export interface IFileSystem {
  /** Lê um arquivo inteiro. */
  readFile(path: string): Promise<Buffer>;
  /** Grava (ou sobrescreve) um arquivo. */
  writeFile(path: string, data: Buffer): Promise<void>;
  /** Cria um diretório temporário novo e devolve o caminho dele. */
  makeTempDir(): Promise<string>;
  /** Remove um diretório e tudo que há dentro; não falha se ele não existir. */
  removeDir(path: string): Promise<void>;
  /** Junta um diretório e um nome de arquivo com o separador da plataforma. */
  joinPath(directory: string, name: string): string;
}

/** Implementação real, sobre `node:fs/promises`. */
export class NodeFileSystem implements IFileSystem {
  /** Lê um arquivo inteiro. */
  readFile(path: string): Promise<Buffer> {
    return readFile(path);
  }

  /** Grava (ou sobrescreve) um arquivo. */
  writeFile(path: string, data: Buffer): Promise<void> {
    return writeFile(path, data);
  }

  /** Cria um diretório temporário novo, dentro do diretório temporário do sistema. */
  makeTempDir(): Promise<string> {
    return mkdtemp(join(tmpdir(), 'thumbcraft-'));
  }

  /** Remove um diretório e tudo que há dentro. */
  removeDir(path: string): Promise<void> {
    return rm(path, { recursive: true, force: true });
  }

  /** Junta um diretório e um nome de arquivo com `path.join`. */
  joinPath(directory: string, name: string): string {
    return join(directory, name);
  }
}
