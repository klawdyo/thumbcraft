# Arquitetura do thumbcraft

O thumbcraft é um binário Rust único: recebe um documento Word, uma planilha ou uma apresentação
e devolve uma imagem (miniatura). Usa os motores de renderização dos três projetos *craft da
organização storytold (WordCraft, GridCraft e DeckCraft) como bibliotecas, sem GUI, MCP nem
servidores. As versões usadas estão em `UPSTREAM.md`.

## Estrutura

```
Cargo.toml                workspace (perfil de release para tamanho, remendo do krilla)
crates/core               thumbcraft-core: sem dependências; tudo que é testável sem motores
  render.rs               trait Render (a porta)
  fake.rs                 impl Fake de Render, para os testes
  route.rs                escolha do motor por conteúdo e por extensão
  opts.rs                 opções e analisadores (fit, gravity, background, crop, filtro, escala)
  scale.rs                escala de renderização automática e limites
  fit.rs                  matemática do ajuste (cover, contain, gravity, uma dimensão)
  trim.rs                 remoção de margens brancas ou transparentes
  resize.rs               reamostragem (área e Lanczos3)
  post.rs                 pipeline: renderiza, recorta, apara, ajusta, reduz
crates/thumbcraft         binário
  word.rs grid.rs deck.rs impls reais de Render (WordCraft, GridCraft, DeckCraft)
  open.rs                 inspeção do arquivo e abertura pelo motor certo
  cli.rs                  análise dos argumentos
  encode.rs               codificação PNG (e JPEG e WebP, ver docs/SIZES.md)
  info.rs build.rs        --version e --formats, com os SHAs embutidos em tempo de build
scripts/                  build, busca dos upstream, execução em corpus, comparação nativa, sheets
patches/gridcraft/        patch do exportador de PNG do GridCraft
```

## A porta `Render`

```rust
pub trait Render {
    fn unit(&self) -> &'static str;                         // "página", "slide"
    fn pages(&self) -> usize;
    fn size(&mut self, page: usize) -> Res<(f64, f64)>;      // pontos (1/72 pol)
    fn render(&mut self, page: usize, scale: f64) -> Res<Raster>; // RGBA pré-multiplicado
    fn native_png(&mut self, page: usize, scale: f64) -> Res<Vec<u8>>; // só para o teste nativo
}
```

Uma implementação real por motor (`Word`, `Grid`, `Deck` e `Pdf`) e uma `Fake` para os testes do
pós-processamento. `Pdf` (`crates/thumbcraft/src/pdf.rs`) usa o Hayro pelo `craft-pdf` com fundo
branco opaco, os limites de entrada e de saída do `craft-pdf` e avisos em português na stderr quando
o leitor omite conteúdo ou não decodifica uma imagem.

Os índices da CLI começam em 1; a porta usa índices a partir de 0 (a conversão é feita em
`post::process`).

### Como cada motor é usado

- **Word**: `open_path_in` abre o arquivo, `Session::export_layout_with(false)` pagina (o mesmo
  layout do comando `file.exportPng`), `render_page` rasteriza. `native_png` usa `Rendered::to_png`,
  o mesmo caminho do comando, e por isso sai idêntico ao wc-min.
- **Grid**: só comandos públicos do engine: `file.open`, `sheet.activate` na primeira aba visível,
  `pageLayout.pageSetup` (A4, sem ajuste à página), `file.printPreview` para saber quantas
  páginas de impressão existem e o tamanho de cada uma, `file.exportPng` (sem `path`, devolve o PNG
  em base64). O PNG é decodificado para o pós-processamento. Há um ciclo de codificar e decodificar
  PNG que custa algumas dezenas de ms; é o preço de não depender de funções internas do GridCraft.
- **Deck**: `import_checked`, `repair`, fontes embutidas no `FontDb`, `render_slide`. Diferença
  deliberada em relação ao dc-min: um slide danificado (XML quebrado) é erro 4, porque o motor o
  descarta e renderizaria o seguinte no lugar. A ordem original dos slides é reconstruída de
  `ppt/presentation.xml` e seus relacionamentos; os slides depois de um danificado continuam
  acessíveis pelo número original.

## Roteamento

1. O conteúdo manda quando é inequívoco: ZIP com `word/`, `xl/` ou `ppt/` (ou, na falta de pastas,
   tipos de conteúdo de `[Content_Types].xml`), `mimetype` ODF (texto, planilha, apresentação),
   `%PDF` (no início ou depois de até 1 KB de lixo), `{\rtf`. Isso tolera extensão errada (um pptx chamado `.docx` vai para o DeckCraft).
2. Senão vale a extensão. Contêiner CFB (formatos antigos e pacotes criptografados) segue a
   extensão. Texto puro com extensão `.pdf` vai para o WordCraft (um markdown salvo como pdf).
3. `.ppt` e `.odp` (inclusive ZIP ODF de apresentação): erro 3 sugerindo converter para pptx.
   Desconhecido: erro 3. PDF vai para o motor de PDF; um arquivo `.pdf` que não é PDF (uma imagem, por
   exemplo) falha ao abrir com erro 4.
4. Um arquivo que o conteúdo não identifica mas cuja extensão é de um motor segue para ele e falha
   ao abrir (erro 4), como nos minis (texto puro chamado `.pptx`, arquivo vazio, truncado).

`thumbcraft --detect ARQUIVO` imprime o motor escolhido (usado por `scripts/run-corpus.sh`).

### Entrada e saída por `-` (stdin e stdout)

- `-` como ENTRADA lê o documento inteiro da stdin, até 256 MiB (`io::MAX_STDIN`); entrada vazia ou acima do
  limite é erro 4, falha ao ler é erro 5. Os motores abrem por caminho, então os bytes vão para uma cópia
  temporária (`io::TempInput`, em uma pasta própria do diretório temporário, apagada ao fim do processo).
- `-` como SAÍDA grava só os bytes da imagem na stdout (também em `--native-scale`); falha ao gravar, como
  stdout fechada, é erro 5. Mensagens continuam só na stderr.
- `--input-name <nome>`: dica de nome; só a extensão importa e ela vai para a cópia temporária, para o roteamento
  de sempre (conteúdo primeiro, extensão depois). Sem a dica decide só o conteúdo e, se ele for silencioso, erro 3.
- `--detect -` lê a stdin do mesmo modo. Diferença de propósito: arquivo vazio em disco ainda é roteado pela extensão
  (e falha ao abrir, erro 4), enquanto stdin vazia já é erro 4 na leitura.
- Com `panic = "abort"` uma falha interna pode deixar a pasta temporária para trás.

## Pipeline de pós-processamento (`post::process`)

1. Valida opções (erro 2) e a existência da página (erro 4).
2. Escolhe a escala de renderização (`scale::choose`). Em `auto`, a renderização tem ao menos o
   dobro do alvo (supersampling de 2x: redução por um fator de pelo menos 2), com no mínimo
   1 px/pt e no máximo 8192 px por lado e 64 milhões de pixels. Sem largura nem altura, usa 2 px/pt
   (a escala dos minis). `--render-scale N` fixa a escala, ainda limitada pelos máximos.
3. Renderiza a página inteira em alta resolução.
4. `--crop x,y,l,a` (frações da página renderizada) vira um retângulo em pixels.
5. `--trim` procura o menor retângulo com conteúdo dentro do recorte (pixel vazio: branco com
   tolerância de 5 níveis ou transparente) e soma `--trim-pad` pixels. Se o conteúdo ocupa pouco e a
   resolução ficou curta para o alvo, renderiza UMA segunda vez na escala que o conteúdo pede
   (comum em planilhas, onde a página é quase toda branca).
6. `fit::plan` calcula o tamanho final e a janela de origem; `resize::resample` reduz a janela
   (fracionária, limitada à região) com Lanczos3 ou média de área, em alfa pré-multiplicado.
7. O fundo (`--background`) é composto; no contain completa a caixa. PNG sai em RGB quando a imagem
   é opaca (menor), RGBA quando há transparência.

Ordem fixa: crop, trim, fit. Sem largura e altura a região sai como está.

## Decisões e padrões propostos (a confirmar com o usuário)

| Tema | Padrão atual | Observação |
|---|---|---|
| `--fit` | `cover` | Em slides 16:9 numa caixa quadrada o cover corta as laterais; `contain` mostra o slide inteiro |
| `--gravity` | `top` no cover, `center` no contain | `top` centraliza na horizontal |
| `--background` | branco | aceita `transparent` e `#RRGGBB` |
| Filtro | Lanczos3 (`--filter area` disponível) | Lanczos é mais nítido; área é mais rápida e gera PNG menor (ver docs/SIZES.md) |
| `--trim-pad` | 8, em pixels da renderização nativa | Com escala `auto` a renderização tem cerca de 2x o alvo, então 8 px nativos são cerca de 4 px finais |
| Ordem | crop, trim, fit | o trim age dentro do recorte |
| Tamanho | `-W` e `-H` | sem `--size LxA` |
| Sem tamanho | escala 2 px/pt, sem redução | como os minis |
| Falha ao gravar a saída | código 5 | não é culpa do arquivo de entrada |
| Texto com extensão `.pdf` | vai para o WordCraft | tolerância a extensão errada |
| Slide danificado no PPTX | erro 4 | o dc-min renderizaria o slide seguinte |
| `--page` | a partir de 1 | slide, página do documento ou página de impressão da planilha |
| Formatos de saída | PNG, e JPEG e WebP se o custo for pequeno | ver docs/SIZES.md |

## Limitações conhecidas

- **Texto não latino no GridCraft**: as 12 fontes embutidas no exportador de PNG cobrem WinAnsi mais
  alguns símbolos; árabe, CJK e emoji viram `?` na planilha.
- **Fontes substituídas**: o WordCraft embute 13 TTF (Source Sans, Source Serif, Inter, JetBrains
  Mono) e usa fontes do sistema quando há; fontes ausentes (Calibri, Arial, Times) são substituídas
  por equivalentes, então quebras de linha e de página podem diferir do Word. O DeckCraft roda
  sem o repositório `craft-fonts` (fica vazio no thumbcraft, como no dc-min) e usa o fallback e as
  fontes do sistema; o resultado depende das fontes instaladas na máquina (um slide com RTL, CJK e
  emoji levou 970 ms uma vez, por varredura das fontes do sistema). Não é determinístico entre
  máquinas com conjuntos de fontes diferentes.
- **Planilhas**: a página é A4 na orientação da própria planilha, sem zoom e sem ajuste à página,
  então planilhas largas geram várias páginas e `--page 1` mostra só o canto superior esquerdo.
  Use `--trim` para aproximar do conteúdo. Região de impressão, quebras e escala da planilha são
  respeitadas. xls e ods são leituras parciais.
- **Locale**: o WordCraft escolhe papel (A4 ou Letter) de documentos sem tamanho (txt, md, html)
  pelas variáveis de ambiente de locale (`LANG`, `LC_*`), como o wc-min. Para saída determinística,
  defina as mesmas variáveis sempre.
- **PowerPoint**: `.ppt` e `.odp` não são lidos pelo DeckCraft. Slides ocultos são renderizados
  como os demais.
- **Documentos com senha** falham ao abrir (erro 4).

## panic = "abort"

O perfil de release usa `panic = "abort"` (menor binário). Uma falha interna de um motor derruba o
processo inteiro (SIGABRT, status de saída 134 no shell) sem mensagem em português e sem arquivo de
saída. O DeckCraft tinha uma rede de proteção por `catch_unwind` que devolvia um slide em branco;
com abort ela não funciona. Por isso o chamador deve:

- usar um processo por arquivo (nunca reaproveitar o processo em lote);
- impor um timeout (os scripts usam 30 s);
- tratar qualquer status diferente de 0 a 5 (por exemplo 134 ou 124 do `timeout`) como falha.

A CLI roda o trabalho numa thread com pilha de 128 MB para que documentos muito aninhados não
estourem a pilha padrão de 8 MB; um panic nessa thread também aborta o processo.

## Códigos de saída

| Código | Significado |
|---|---|
| 0 | ok |
| 2 | uso incorreto (opção inválida, valor fora do limite) |
| 3 | formato não suportado (`.ppt`, `.odp`, extensão desconhecida) |
| 4 | arquivo inválido ou falha ao abrir ou renderizar (inclui página inexistente e slide danificado) |
| 5 | erro interno (inclui falha ao gravar a saída) |

Mensagens de erro em português na stderr; nada na stdout (exceto `--version`, `--formats`,
`--help` e `--detect`, que são pedidos de informação, e a imagem quando a saída é `-`). O texto de erro do motor, depois do
prefixo em português, vem em inglês como o motor o produziu.

## Testes

- `cargo test` (raiz): 80 testes do núcleo, rápidos, usando a `Fake` (matemática do fit, cover,
  contain, gravity, trim, crop, uma dimensão, caixas degeneradas, limites, escala automática,
  reamostragem, roteamento por extensão e por conteúdo).
- `cargo test --workspace`: 84 testes no total, inclui a análise de argumentos do binário (4 testes) (compila os motores em perfil
  de teste, demorado).
- `scripts/compare-native.sh`: prova de não regressão contra wc-min, dc-min e gc-min.
