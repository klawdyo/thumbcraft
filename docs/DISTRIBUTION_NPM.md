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
        └── @thumbcraft/linux-x64-gnu     ──> bin/thumbcraft
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
    "libc": ["glibc"]
  }
}
```

**Benefício prático:** Ao rodar `npm install` no Windows, o gerenciador de pacotes baixa tanto o binário do Windows quanto o do Linux (`glibc`, para Ubuntu/Debian). O `package-lock.json` registra ambos de forma determinística, permitindo testar o código no Windows e montar a pasta em um container Docker sem falhas de resolução ou necessidade de reconstruir o lockfile no WSL.

---

## 3. Matriz de Plataformas e Pacotes Satélites

Só Windows x64 e Linux x64 (glibc) são suportados; macOS, ARM e Alpine (musl) estão fora do escopo por decisão do dono (2026-10-19). Cada pacote satélite declara estritamente seu ambiente-alvo para que o instalador do npm descarte os binários irrelevantes para aquele sistema:

| Nome do Pacote Satélite | Sistema | Arquitetura | Libc | Cenário de Uso |
| :--- | :--- | :--- | :--- | :--- |
| `@thumbcraft/win32-x64-msvc` | `win32` | `x64` | N/A | Windows 10/11 / Windows Server |
| `@thumbcraft/linux-x64-gnu` | `linux` | `x64` | `glibc` | Ubuntu 22/24, Debian, CentOS, Fedora |

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

### 4.1 Validação prévia (`Thumbcraft.isValid`)
Decide se o arquivo é suportado **sem duplicar o roteamento em TypeScript**: a função chama `thumbcraft --detect`, que usa exatamente a mesma lógica do binário (`route.rs` e `open.rs`), então o resultado nunca diverge dele. Por isso é assíncrona e custa a criação de um processo (dezenas de milissegundos), não nanossegundos; não há leitura de ZIP nem tabela de assinaturas no código TypeScript.

```typescript
import { Thumbcraft } from 'thumbcraft';

// Retorna o motor que trataria o arquivo ('word', 'grid' ou 'deck') ou null se não for suportado.
const engine = await Thumbcraft.detectFile('/caminho/do/documento.docx');

// Atalhos booleanos
if (await Thumbcraft.isValidFile('/caminho/do/documento.docx')) {
  // ...
}
if (await Thumbcraft.isValid(fileBuffer, originalFilename)) {
  // Buffer: usa a entrada por stdin (tarefa 01) com a dica de nome para o roteamento
}
```

### 4.2 Métodos de Entrada e Saída (Arquivo <-> Buffer)

A classe `Thumbcraft` oferece uma interface fluente com suporte a entrada por arquivo ou por buffer:

```typescript
import { Thumbcraft, RenderOptions } from 'thumbcraft';

// 1. Buffer -> Buffer (Ideal para pipelines de upload em memória: Multer -> Thumbcraft -> Sharp)
const imageBuffer: Buffer = await Thumbcraft
  .fromBuffer(uploadedFileBuffer, 'relatorio.xlsx')
  .toBuffer({ scale: 2, trim: true });

// 2. Buffer -> Arquivo (Grava diretamente em disco)
await Thumbcraft
  .fromBuffer(uploadedFileBuffer, 'apresentacao.pptx')
  .toFile('./thumbnails/slide1.png', { fit: 'contain', width: 800, height: 600 });

// 3. Arquivo -> Buffer
const pngBuffer: Buffer = await Thumbcraft
  .fromFile('./contrato.docx')
  .toBuffer({ page: 1 });

// 4. Arquivo -> Arquivo
await Thumbcraft
  .fromFile('./planilha.ods')
  .toFile('./saida.png');
```

### 4.3 Opções de Renderização (`RenderOptions`)
Alinhadas às opções de pós-processamento do núcleo (`crates/core/src/opts.rs`):

```typescript
export interface RenderOptions {
  /** Número da página ou slide a renderizar (a partir de 1; padrão: 1) */
  page?: number;

  /** Escala nativa de renderização em px por pt, ou 'auto' (padrão: 'auto', cerca de 2x o tamanho pedido) */
  scale?: number | 'auto';

  /** Recorta margens vazias automaticamente detectando a cor de fundo (padrão: false) */
  trim?: boolean;

  /** Margem mantida em volta do conteúdo pelo trim, em pixels da renderização nativa (padrão: 8) */
  trimPad?: number;

  /** Como a imagem deve se ajustar à caixa de largura/altura */
  fit?: 'contain' | 'cover';

  /** Ponto de âncora para cortes ou ajustes (padrão: 'top' com fit 'cover' e 'center' com fit 'contain') */
  gravity?: 'top' | 'bottom' | 'left' | 'right' | 'center' | 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right';

  /** Largura máxima da imagem final em pixels */
  width?: number;

  /** Altura máxima da imagem final em pixels */
  height?: number;

  /** Fundo do contain e sob transparência: 'white' (padrão), 'transparent' ou "#RRGGBB" */
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
| `trimPad: 8` | `--trim-pad 8` | Margem mantida pelo trim em pixels nativos |
| `gravity: 'top'` | `--gravity top` | Âncora de corte/enquadramento (`top` no cover e `center` no contain, quando omitida) |
| `background: '#FFFFFF'` | `--background #FFFFFF` | Fundo (`white`, `transparent` ou `#RRGGBB`) |
| `page: 1` | `--page 1` | Número da página/slide a renderizar (a partir de 1) |
| `scale: 2.0` | `--render-scale 2.0` | Escala do rasterizador nativo em px por pt (ou `'auto'`, o padrão) |
| `format: 'webp'` | `--format webp` | Formato da saída (`png`, `jpeg` ou `webp`) |
| `quality: 85` | `--quality 85` | Qualidade do JPEG (1 a 100) |

**Exemplo de geração direta sem Sharp:**
```typescript
// Gera o thumbnail final direto, no tamanho e recorte exatos, 100% autônomo
const finalThumbnail = await Thumbcraft
  .fromBuffer(fileBuffer, 'contrato.docx')
  .toBuffer({
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
  canProcess(file: UploadedFileMetadata): Promise<boolean>;
  process(file: UploadedFileMetadata, targetWidth?: number, targetHeight?: number): Promise<Buffer>;
}

/**
 * Estratégia de documentos de escritório usando o Thumbcraft
 * Renderiza DOCX, XLSX, PPTX, ODT, ODS e converte o resultado em buffer PNG,
 * repassando-o diretamente ao Sharp para redimensionamento e otimização final.
 */
@Injectable()
export class OfficeDocumentProcessingStrategy implements ProcessingStrategy {
  async canProcess(file: UploadedFileMetadata): Promise<boolean> {
    return Thumbcraft.isValid(file.buffer, file.originalname);
  }

  async process(file: UploadedFileMetadata, targetWidth = 320, targetHeight = 320): Promise<Buffer> {
    // 1. Gera o screenshot de alta resolução da primeira página usando o binário unificado
    const rawScreenshotBuffer = await Thumbcraft
      .fromBuffer(file.buffer, file.originalname)
      .toBuffer({ scale: 1.5, trim: true });

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

Decisão do dono (2026-10-19): a publicação é **automática a cada push na `main`**, sem tag e **sem token do npm**, usando a publicação confiável (trusted publishing, OIDC) do npm.

Como o npm recusa publicar duas vezes a mesma versão, o workflow se comporta assim a cada push na `main`:

1. **Sempre** roda os testes (Windows e Ubuntu). Em pull requests, só isso.
2. Lê a versão do `Cargo.toml` (`[workspace.package] version`) e consulta o npm. Se essa versão **ainda não está publicada**, compila os binários, monta os três pacotes e publica. Se já está, termina sem publicar.

Na prática, a publicação acontece quando entra na `main` o commit `release: X.Y.Z` criado pelo `versioner`, e só então. Os demais commits só rodam os testes. Publicar a cada commit exigiria que o CI aumentasse a versão sozinho, o que contraria a regra de que a versão só muda pelo `versioner`, a pedido do dono; fica como possível evolução, só com autorização dele.

Passos do job de publicação:

1. Baixar os motores (`scripts/fetch-upstream.sh`) e compilar o binário em release para Windows x64 e Linux x64 (`opt-level = "z"`, `lto = "fat"`, `strip = "symbols"`).
2. Montar os três pacotes: `@thumbcraft/win32-x64-msvc`, `@thumbcraft/linux-x64-gnu` e `thumbcraft`, todos com a versão do `Cargo.toml`.
3. Publicar **primeiro os dois pacotes de plataforma e por último o principal**, porque o principal depende deles.

Requisitos da publicação confiável (documentação do npm): npm 11.5.1 ou mais novo, Node 22.14 ou mais novo, `id-token: write` nas permissões do job, runner hospedado pelo GitHub, e cada pacote configurado no npm com o repositório e o **nome do arquivo do workflow** (`release.yml`). O campo `repository.url` do `package.json` de cada pacote precisa ser exatamente o do repositório no GitHub. Não há segredo nem `NODE_AUTH_TOKEN`; a proveniência é gerada automaticamente.

### Exemplo de Workflow: `.github/workflows/release.yml` (rascunho, só Windows x64 e Linux x64)

```yaml
name: Release

on:
  push:
    branches: [main]
  pull_request:

permissions:
  contents: read

jobs:
  test:
    name: Test (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [windows-latest, ubuntu-24.04]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/setup-node@v4
        with:
          node-version: 22
      - name: Baixar motores upstream e aplicar patches
        shell: bash
        run: ./scripts/fetch-upstream.sh
      - run: cargo test --workspace --locked
      - name: Testar o wrapper
        working-directory: packages/thumbcraft
        run: npm ci && npm test

  check-version:
    name: Check version
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    needs: test
    runs-on: ubuntu-24.04
    outputs:
      version: ${{ steps.version.outputs.version }}
      publish: ${{ steps.published.outputs.publish }}
    steps:
      - uses: actions/checkout@v4
      - id: version
        name: Ler a versão do Cargo.toml
        run: |
          VERSION=$(grep -m1 '^version = ' Cargo.toml | sed -E 's/version = "(.*)"/\1/')
          echo "version=$VERSION" >> "$GITHUB_OUTPUT"
      - id: published
        name: Consultar o npm
        run: |
          if npm view "thumbcraft@${{ steps.version.outputs.version }}" version >/dev/null 2>&1; then
            echo "publish=false" >> "$GITHUB_OUTPUT"
          else
            echo "publish=true" >> "$GITHUB_OUTPUT"
          fi

  build-binaries:
    name: Build (${{ matrix.target }})
    needs: check-version
    if: needs.check-version.outputs.publish == 'true'
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            binary_name: thumbcraft.exe
          - os: ubuntu-24.04
            target: x86_64-unknown-linux-gnu
            binary_name: thumbcraft
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - name: Baixar motores upstream e aplicar patches
        shell: bash
        run: ./scripts/fetch-upstream.sh
      - run: cargo build --release --locked --target ${{ matrix.target }} -p thumbcraft
      - uses: actions/upload-artifact@v4
        with:
          name: binary-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/${{ matrix.binary_name }}

  publish:
    name: Publish to npm
    needs: [check-version, build-binaries]
    runs-on: ubuntu-24.04
    permissions:
      contents: read
      id-token: write # exigido pela publicação confiável (OIDC)
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 22
          registry-url: https://registry.npmjs.org
      - name: Garantir npm 11.5.1 ou mais novo
        run: npm install -g npm@latest
      - uses: actions/download-artifact@v4
        with:
          path: artifacts/
      - name: Montar os pacotes
        working-directory: packages/thumbcraft
        env:
          VERSION: ${{ needs.check-version.outputs.version }}
        run: |
          npm ci
          npm run build
          npm run build:packages -- --version "$VERSION" \
            --binary win32-x64-msvc=../../artifacts/binary-x86_64-pc-windows-msvc/thumbcraft.exe \
            --binary linux-x64-gnu=../../artifacts/binary-x86_64-unknown-linux-gnu/thumbcraft
      - name: Publicar os pacotes de plataforma e depois o principal
        run: |
          npm publish ./outputs/packages/win32-x64-msvc --access public
          npm publish ./outputs/packages/linux-x64-gnu --access public
          npm publish ./outputs/packages/thumbcraft --access public
```

> O rascunho usa o nome `release.yml`; se o arquivo tiver outro nome, o nome configurado em cada pacote no npm precisa ser o mesmo. O script `npm run build:packages` (em `packages/thumbcraft/tools/`) e os modelos em `packages/platforms/` já existem (tarefa 04); os pacotes montados ficam em `outputs/packages/`.

## 7. Próximos Passos de Implementação

1. **Criar a pasta `packages/` no repositório:**
   - `packages/thumbcraft/`: wrapper TypeScript e definições de tipo (já existe).
   - `packages/platforms/*`: um pacote por plataforma, com o binário (tarefa 04).
2. **Implementar o suporte a stdin e stdout no CLI do Rust:**
   - Permitir `thumbcraft - -` para ler bytes de documento via `stdin` e gravar a imagem direto na `stdout`, viabilizando chamadas em memória com streams do Node.js (tarefa 01).
3. **Preparar o npm (uma vez, pelo dono):** seguir a seção 8.
4. **Criar o workflow `release.yml`** da seção 6 (tarefa 06).

---

## 8. Passo a passo no npm para a publicação confiável (feito pelo dono, uma vez)

A publicação confiável **não cria pacotes novos**: a primeira versão de cada pacote precisa ser publicada à mão, e só depois o npm é configurado para confiar no workflow do GitHub. Essa limitação vem de guias de terceiros ([exemplo](https://developer.wordpress.org/playground/handbook/contributing/releases/)); a [documentação oficial](https://docs.npmjs.com/trusted-publishers) não a descreve, então confirme na tela de configuração do pacote. Não há segredo para guardar no GitHub.

1. **Conta e organização.** Crie a conta no npmjs.com (com autenticação em dois fatores) e crie a organização `thumbcraft` (o escopo `@thumbcraft`), gratuita para pacotes públicos.
2. **Primeira publicação manual dos três pacotes**, a partir da sua máquina, depois que a tarefa 04 gerar os pacotes (a ordem importa: plataformas primeiro, principal por último):
   ```sh
   npm login
   npm publish ./packages/platforms/win32-x64-msvc --access public
   npm publish ./packages/platforms/linux-x64-gnu --access public
   npm publish ./packages/thumbcraft --access public
   ```
   Use a versão atual do `Cargo.toml`. Essa publicação manual é a única que usa a sua conta; as seguintes serão do CI. Depois dela, o CI só publica versões **maiores**.
3. **Configurar cada um dos três pacotes** no npmjs.com: página do pacote, Settings, "Trusted Publisher", GitHub Actions, com:
   - Organization or user: `klawdyo`
   - Repository: `thumbcraft`
   - Workflow filename: `release.yml` (com a extensão e sem pasta)
   - Environment: em branco
   - Allowed actions: marque `npm publish`

   Os campos diferenciam maiúsculas de minúsculas. O npm não valida a configuração ao salvar: um erro só aparece na hora de publicar. Uma configuração nova expira se não houver uma publicação bem-sucedida em 2 dias. O campo `repository.url` do `package.json` de cada pacote deve ser exatamente o do repositório no GitHub (`git+https://github.com/klawdyo/thumbcraft.git`).
4. **Opcional, por linha de comando:** versões novas do npm (11.15 ou mais nova, segundo um projeto de terceiros, não confirmado na documentação oficial) têm `npm trust github` para fazer o passo 3 sem a interface.
5. **Depois que o primeiro release automático funcionar:** nas configurações de cada pacote, restrinja a publicação por token e revogue qualquer token temporário usado na primeira publicação. O segredo `NPM_THUMBCRAFT` que você criou no GitHub deixa de ser necessário e pode ser apagado.
6. **Para fazer um release:** peça ao agente para rodar o `versioner` (ele cria o commit `release: X.Y.Z`); em seguida rode `cargo update --workspace` e commite o `Cargo.lock` à parte, para o build com `--locked` não falhar; faça o push dos dois commits na `main`, e o workflow publica sozinho.
