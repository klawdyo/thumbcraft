# Arquitetura de Distribuição NPM e Pipeline CI/CD: `thumbcraft`

Este documento detalha o plano arquitetural para publicar e manter o **`thumbcraft`** no registro oficial do [npm](https://www.npmjs.com) como uma biblioteca Node.js / TypeScript com binários nativos pré-compilados e suporte multiplataforma.

> **Status do nome:** O pacote `thumbcraft` encontra-se **disponível e livre** no registro do npm.

---

## 1. Visão Geral da Arquitetura

Para garantir velocidade de execução (80–120 ms por miniatura) sem impor compilação na máquina do usuário (`node-gyp` ou ferramentas C++), adota-se o mesmo padrão utilizado por projetos de alta performance como `sharp`, `esbuild` e `swc`:

1. **Pacote Raiz (`thumbcraft`):** Código TypeScript/JavaScript leve que provê a API de alto nível, validação de arquivos por sniffing de bytes e orquestração dos binários.
2. **Subpacotes de Plataforma (`@thumbcraft/*`):** Pacotes satélites pequenos que contêm exclusivamente o executável estático compilado pelo Rust para cada arquitetura e sistema operacional.
3. **Download Seletivo:** O npm baixa automaticamente **apenas** o binário correspondente ao ambiente do usuário através de `optionalDependencies` com as diretivas `os`, `cpu` e `libc`.

```
thumbcraft (pacote principal instalado pelo usuário)
  ├── package.json (declara optionalDependencies)
  ├── index.js / index.d.ts (API fluente: fromFile, fromBuffer, isValid)
  └── bin/thumbcraft (script wrapper que resolve o caminho do executável)
        │
        ├── @thumbcraft/win32-x64-msvc   ──> bin/thumbcraft.exe
        ├── @thumbcraft/linux-x64-gnu     ──> bin/thumbcraft
        ├── @thumbcraft/linux-x64-musl    ──> bin/thumbcraft
        ├── @thumbcraft/darwin-arm64      ──> bin/thumbcraft
        └── @thumbcraft/darwin-x64        ──> bin/thumbcraft
```

---

## 2. Suporte Cross-Platform no Desenvolvimento Local (`supportedArchitectures`)

Um dos maiores problemas no desenvolvimento com binários nativos é a disparidade entre o ambiente de desenvolvimento (ex.: Windows) e o ambiente de produção (ex.: containers Docker baseados em Debian ou Alpine).

Com o suporte nativo do **npm 10+**, qualquer aplicação que utilize o `thumbcraft` pode declarar a chave `supportedArchitectures` no seu `package.json`:

```json
{
  "name": "minha-aplicacao",
  "supportedArchitectures": {
    "os": ["win32", "linux"],
    "cpu": ["x64"],
    "libc": ["glibc", "musl"]
  }
}
```

**Benefício prático:** Ao rodar `npm install` no Windows, o gerenciador de pacotes baixa tanto o binário do Windows quanto os binários do Linux (`glibc` para Ubuntu/Debian e `musl` para Alpine). O `package-lock.json` registra ambos de forma determinística, permitindo testar o código no Windows e montar a pasta em um container Docker sem falhas de resolução ou necessidade de reconstruir o lockfile no WSL.

---

## 3. Matriz de Plataformas e Pacotes Satélites

Cada pacote satélite declara estritamente seu ambiente-alvo para que o instalador do npm descarte os binários irrelevantes para aquele sistema:

| Nome do Pacote Satélite | Sistema | Arquitetura | Libc | Cenário de Uso |
| :--- | :--- | :--- | :--- | :--- |
| `@thumbcraft/win32-x64-msvc` | `win32` | `x64` | N/A | Windows 10/11 / Windows Server |
| `@thumbcraft/linux-x64-gnu` | `linux` | `x64` | `glibc` | Ubuntu 22/24, Debian, CentOS, Fedora |
| `@thumbcraft/linux-x64-musl` | `linux` | `x64` | `musl` | Alpine Linux (containers enxutos) |
| `@thumbcraft/darwin-arm64` | `darwin` | `arm64` | N/A | macOS Apple Silicon (M1/M2/M3/M4) |
| `@thumbcraft/darwin-x64` | `darwin` | `x64` | N/A | macOS Intel |

### Estrutura do `package.json` do pacote satélite (exemplo Linux GNU):
```json
{
  "name": "@thumbcraft/linux-x64-gnu",
  "version": "0.1.0",
  "description": "Binário nativo do Thumbcraft para Linux x64 (glibc)",
  "os": ["linux"],
  "cpu": ["x64"],
  "libc": ["glibc"],
  "bin": {
    "thumbcraft": "bin/thumbcraft"
  },
  "files": [
    "bin/thumbcraft"
  ]
}
```

---

## 4. Design da API Node.js / TypeScript (`thumbcraft`)

A API da biblioteca foi desenhada para se integrar com fricção zero em backends modernos, frameworks (NestJS, Express, Fastify) e bibliotecas de processamento de imagem como o **`sharp`**, suportando tanto arquivos no sistema de arquivos quanto buffers em memória.

### 4.1 Validação Prévia Rápida (`Thumbcraft.isValid`)
Executa a lógica de *sniffing* dos primeiros bytes (assinatura mágica de ZIP OOXML, CFB para `.doc`/`.xls`, ODF ou RTF) em nanossegundos, espelhando a lógica de roteamento do `route.rs`. Isso permite que a aplicação decida se deve acionar o Thumbcraft antes mesmo de invocar o processo.

```typescript
import { Thumbcraft } from 'thumbcraft';

// Validação por Buffer (inspeção dos cabeçalhos binários + nome opcional)
if (Thumbcraft.isValid(fileBuffer, originalFilename)) {
  // Arquivo suportado (Word, Excel, PowerPoint, ODF, RTF, etc.)
}

// Validação por caminho de arquivo
if (Thumbcraft.isValidFile('/caminho/do/documento.docx')) {
  // ...
}
```

### 4.2 Métodos de Entrada e Saída (Arquivo <-> Buffer)

A classe `Thumbcraft` oferece uma interface fluente com suporte a entrada por arquivo ou por buffer:

```typescript
import { Thumbcraft, RenderOptions } from 'thumbcraft';

// 1. Buffer -> Buffer (Ideal para pipelines de upload em memória: Multer -> Thumbcraft -> Sharp)
const imageBuffer: Buffer = await Thumbcraft
  .fromBuffer(uploadedFileBuffer, 'relatorio.xlsx')
  .toImageBytes({ scale: 2, trim: true });

// 2. Buffer -> Arquivo (Grava diretamente em disco)
await Thumbcraft
  .fromBuffer(uploadedFileBuffer, 'apresentacao.pptx')
  .toImageFile('./thumbnails/slide1.png', { fit: 'contain', width: 800, height: 600 });

// 3. Arquivo -> Buffer
const pngBuffer: Buffer = await Thumbcraft
  .fromFile('./contrato.docx')
  .toImageBytes({ page: 1 });

// 4. Arquivo -> Arquivo
await Thumbcraft
  .fromFile('./planilha.ods')
  .toImageFile('./saida.png');
```

### 4.3 Opções de Renderização (`RenderOptions`)
Alinhadas às opções de pós-processamento do núcleo (`crates/core/src/opts.rs`):

```typescript
export interface RenderOptions {
  /** Número da página ou slide a renderizar (a partir de 1; padrão: 1) */
  page?: number;

  /** Fator de escala/resolução (px por pt; padrão: 1.0) */
  scale?: number;

  /** Recorta margens vazias automaticamente detectando a cor de fundo (padrão: false) */
  trim?: boolean;

  /** Tolerância para o recorte de margens (0.0 a 1.0; padrão: 0.0) */
  trimTolerance?: number;

  /** Como a imagem deve se ajustar à caixa de largura/altura */
  fit?: 'contain' | 'cover';

  /** Ponto de âncora para cortes ou ajustes (padrão: 'center') */
  gravity?: 'top' | 'bottom' | 'left' | 'right' | 'center' | 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right';

  /** Largura máxima da imagem final em pixels */
  width?: number;

  /** Altura máxima da imagem final em pixels */
  height?: number;

  /** Cor de fundo no formato hexadecimal "#RRGGBB" para transparências */
  background?: string;

  /** Formato de saída da imagem (padrão: inferido pela extensão ou 'png') */
  format?: 'png' | 'jpeg' | 'webp';

  /** Qualidade da compressão JPEG (1 a 100, padrão: 85) */
  quality?: number;
}
```

### 4.4 Autossuficiência e Mapeamento para as Flags do Binário

O `thumbcraft` é **100% autossuficiente** para gerar miniaturas completas e ajustadas, **sem necessitar de Sharp ou qualquer biblioteca externa de redimensionamento**.

Todas as propriedades de `RenderOptions` mapeiam diretamente para as flags de linha de comando processadas pelo `thumbcraft-core` e pelo CLI:

| Propriedade JS/TS | Flag no Binário CLI | Descrição |
| :--- | :--- | :--- |
| `width: 320` | `-W, --width 320` | Largura máxima final em pixels |
| `height: 320` | `-H, --height 320` | Altura máxima final em pixels |
| `fit: 'contain'` | `--fit contain` | Encaixa inteiramente (`contain`) ou preenche cortando (`cover`) |
| `trim: true` | `--trim` | Recorta automaticamente margens em branco/vazias |
| `trimTolerance: 0.1` | `--trim-pad 8` | Margem mantida pelo trim em pixels nativos |
| `gravity: 'center'` | `--gravity center` | Âncora de corte/enquadramento (`center`, `top`, etc.) |
| `background: '#FFF'` | `--background white` | Cor de fundo para transparências e preenchimento |
| `page: 1` | `--page 1` | Número da página/slide a renderizar (a partir de 1) |
| `scale: 2.0` | `--render-scale 2.0` | Fator de resolução do rasterizador nativo (ou `auto`) |
| `format: 'webp'` | `--format webp` | Formato da saída (`png`, `jpeg` ou `webp`) |
| `quality: 85` | `--quality 85` | Qualidade do JPEG (1 a 100) |

**Exemplo de geração direta sem Sharp:**
```typescript
// Gera o thumbnail final direto, no tamanho e recorte exatos, 100% autônomo
const finalThumbnail = await Thumbcraft
  .fromBuffer(fileBuffer, 'contrato.docx')
  .toImageBytes({
    width: 300,
    height: 300,
    fit: 'contain',
    trim: true,
  });
```

---

## 5. Exemplo de Integração em Pipeline de Upload (Opcional, com `sharp`)

Caso o backend já utilize o Sharp para padronizar todas as imagens da aplicação (por exemplo, convertendo para WebP/AVIF e aplicando filtros adicionais), o `thumbcraft` pode atuar gerando o screenshot inicial em alta resolução para a esteira:

```typescript
import { Injectable } from '@nestjs/common';
import { Thumbcraft } from 'thumbcraft';
import sharp from 'sharp';

export interface UploadedFileMetadata {
  originalname: string;
  mimetype: string;
  buffer: Buffer;
}

export interface ProcessingStrategy {
  canProcess(file: UploadedFileMetadata): boolean;
  process(file: UploadedFileMetadata, targetWidth?: number, targetHeight?: number): Promise<Buffer>;
}

/**
 * Estratégia de documentos de escritório usando o Thumbcraft
 * Renderiza DOCX, XLSX, PPTX, ODT, ODS e converte o resultado em buffer PNG,
 * repassando-o diretamente ao Sharp para redimensionamento e otimização final.
 */
@Injectable()
export class OfficeDocumentProcessingStrategy implements ProcessingStrategy {
  canProcess(file: UploadedFileMetadata): boolean {
    return Thumbcraft.isValid(file.buffer, file.originalname);
  }

  async process(file: UploadedFileMetadata, targetWidth = 320, targetHeight = 320): Promise<Buffer> {
    // 1. Gera o screenshot de alta resolução da primeira página usando o binário unificado
    const rawScreenshotBuffer = await Thumbcraft
      .fromBuffer(file.buffer, file.originalname)
      .toImageBytes({ scale: 1.5, trim: true });

    // 2. Alimenta o buffer gerado diretamente no pipeline do Sharp
    const optimizedThumbnail = await sharp(rawScreenshotBuffer)
      .resize({
        width: targetWidth,
        height: targetHeight,
        fit: 'inside',
        withoutEnlargement: true,
      })
      .webp({ quality: 85 })
      .toBuffer();

    return optimizedThumbnail;
  }
}
```

---

## 6. Automação de CI/CD (GitHub Actions)

Para evitar manutenção manual de versões em múltiplos pacotes, o workflow do GitHub Actions é acionado na criação de uma tag de versão (ex.: `v0.1.0`) e se encarrega de:

1. Baixar os motores upstream (`scripts/fetch-upstream.sh`).
2. Aplicar os patches necessários (ex.: exportador PNG do GridCraft).
3. Compilar os binários em matriz para todos os sistemas operacionais com perfil otimizado (`opt-level = "z"`, `lto = "fat"`, `strip = "symbols"`).
4. Gerar a estrutura de cada subpacote com o respectivo binário.
5. Publicar automaticamente os subpacotes satélites no npm.
6. Atualizar a referência de dependências no pacote principal `thumbcraft` e publicá-lo.

### Exemplo de Workflow: `.github/workflows/release-npm.yml`

```yaml
name: Release NPM

on:
  push:
    tags:
      - 'v*.*.*'

permissions:
  contents: write
  id-token: write

jobs:
  build-binaries:
    name: Build (${{ matrix.target }})
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            pkg_name: '@thumbcraft/win32-x64-msvc'
            binary_name: thumbcraft.exe
          - os: ubuntu-24.04
            target: x86_64-unknown-linux-gnu
            pkg_name: '@thumbcraft/linux-x64-gnu'
            binary_name: thumbcraft
          - os: ubuntu-24.04
            target: x86_64-unknown-linux-musl
            pkg_name: '@thumbcraft/linux-x64-musl'
            binary_name: thumbcraft
          - os: macos-latest
            target: aarch64-apple-darwin
            pkg_name: '@thumbcraft/darwin-arm64'
            binary_name: thumbcraft

    steps:
      - name: Checkout repositório
        uses: actions/checkout@v4

      - name: Instalar toolchain Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Instalar dependências musl (quando aplicável)
        if: matrix.target == 'x86_64-unknown-linux-musl'
        run: sudo apt-get update && sudo apt-get install -y musl-tools

      - name: Baixar motores upstream e aplicar patches
        shell: bash
        run: ./scripts/fetch-upstream.sh

      - name: Compilar binário de release
        run: cargo build --release --target ${{ matrix.target }} --bin thumbcraft

      - name: Preparar artefato do binário
        shell: bash
        run: |
          mkdir -p dist/bin
          cp target/${{ matrix.target }}/release/${{ matrix.binary_name }} dist/bin/

      - name: Upload do binário compilado
        uses: actions/upload-artifact@v4
        with:
          name: binary-${{ matrix.target }}
          path: dist/bin/

  publish-npm:
    name: Publicar Pacotes no NPM
    needs: build-binaries
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repositório
        uses: actions/checkout@v4

      - name: Configurar Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 20
          registry-url: 'https://registry.npmjs.org'

      - name: Download de todos os binários compilados
        uses: actions/download-artifact@v4
        with:
          path: artifacts/

      - name: Extrair versão da tag Git
        id: get_version
        run: echo "VERSION=${GITHUB_REF_NAME#v}" >> $GITHUB_OUTPUT

      - name: Empacotar e publicar subpacotes satélites
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
          VERSION: ${{ steps.get_version.outputs.VERSION }}
        run: |
          npm run build:packages -- --version "$VERSION"
          npm run publish:satellites

      - name: Publicar pacote principal (thumbcraft)
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
          VERSION: ${{ steps.get_version.outputs.VERSION }}
        run: |
          npm run build:core -- --version "$VERSION"
          cd packages/thumbcraft
          npm publish --access public --provenance
```

---

## 7. Próximos Passos de Implementação

1. **Criar a pasta `npm/` ou `packages/` no repositório:**
   - `packages/thumbcraft/`: Código do wrapper TypeScript e definições de tipo.
   - `packages/platforms/*`: Modelos de `package.json` para cada satélite.
2. **Implementar o suporte a Stdin/Stdout no CLI do Rust:**
   - Permitir `thumbcraft - -` para ler bytes de documento via `stdin` e cuspir o PNG resultante diretamente no `stdout`, viabilizando chamadas 100% em memória através de streams do Node.js.
3. **Configurar o segredo `NPM_TOKEN`:**
   - Adicionar o token com permissão de publicação granular nas configurações de segredos do repositório GitHub.
