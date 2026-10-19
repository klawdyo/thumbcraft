# Commits e versionamento

## Formato (Conventional Commits)

```
<tipo>(<escopo opcional>): <descrição em português>

[corpo opcional]
```

| Tipo | Efeito de versão | Uso |
|------|------------------|-----|
| `feat` | minor | Funcionalidade nova ou mudança grande |
| `fix` | patch | Correção, ajuste ou melhoria em algo existente |
| `docs` | nenhum | Documentação, specs, tarefas |
| `test` | nenhum | Testes novos ou ajustados |
| `refactor` | nenhum | Refatoração sem mudança de comportamento |
| `chore` | nenhum | Manutenção, tooling |
| `style` | nenhum | Formatação, lint |
| `ci` | nenhum | CI/CD |
| `perf` | nenhum | Melhoria de desempenho (reconhecido pelo `versioner`, sem bump) |

## Breaking change — proibido sem autorização expressa

**Nenhum agente pode gerar commit que indique breaking change sem a
autorização expressa do dono, para aquele pedido específico.** Isso inclui:

- `!` depois do tipo/escopo (`feat!:`, `fix(api)!:`);
- o texto `BREAKING CHANGE:` no corpo ou rodapé — **nem como citação, exemplo
  ou explicação** (o `versioner` procura a string e dispara major);
- qualquer outro modo de sinalizar quebra de compatibilidade.

Autorização anterior não vale para o próximo commit. Se a implementação
parece exigir breaking change e ninguém pediu: **pare, descreva o impacto e
pergunte**; não commite como breaking por conta própria nem esconda a quebra
num `fix`/`feat` comum. Ao descrever uma quebra em texto de commit **já
autorizado**, siga o formato da seção de versionamento.

## Atomicidade — o menor commit possível

- **Todo commit é atômico e o menor possível**: um único propósito, contendo
  somente o que mudou para ele. Se dá para dividir, divida.
- **O mesmo arquivo pode (e deve) aparecer em vários commits** quando recebe
  alterações de propósitos diferentes. Separe por hunk com `git add -p` (e
  `e` para editar o hunk quando necessário), nunca commite o arquivo inteiro
  só porque ele "mudou".
- Antes de commitar, confira `git diff --cached`: só entra o que pertence ao
  propósito do commit. Formatação, renomeação e refatoração incidentais vão em
  commits próprios (`style`, `refactor`).
- **Nunca `git add -A` nem `git add .`**: liste arquivos/hunks.
- Teste e implementação correspondente podem ser commits separados
  (`test:` e `feat:`/`fix:`), um por peça.
- Ordem: tipos/contratos compartilhados → implementação → testes →
  integração → docs.
- Mensagem curta no assunto, descrição em português dizendo *o que* mudou;
  corpo só quando o *porquê* não é óbvio.

## Versionamento (`versioner`)

CLI em Go (código em `F:/dev/Go/versioner`, instalado no PATH). Comportamento
**conferido no código-fonte**; não prometa nada além disto.

**Nunca editar a versão à mão** (`package.json`, `pubspec.yaml`) nem fazer
commit `chore(release):`. A versão só muda pelo `versioner`, e **só sob
pedido explícito do dono**.

### Comandos

```bash
versioner calc      # mostra "Base: X.Y.Z" e "Next: X.Y.Z" (ou "Nenhum incremento necessário.")
versioner commit    # atualiza manifestos, faz `git add` deles e cria o commit `release: X.Y.Z`
versioner -v        # versão do próprio versioner
versioner -debug calc
```

- **Não cria tag.** Só altera os manifestos e cria **um commit**
  `release: X.Y.Z`. Tag, push e release são com o dono, manualmente.
- Flags vêm **antes** do comando (`versioner -force commit`); depois do
  comando elas são ignoradas (limitação do pacote `flag` do Go).
- `-force` força um bump `patch` quando nenhum commit gerou incremento
  (apesar de o README dizer que é ignorada).
- Rode na raiz do repositório (ou dentro dele): usa `git rev-parse
  --show-toplevel`. Fora de repositório: erro.
- Sem commit de incremento, `commit` imprime "Nada a versionar." e não faz
  nada.

### Como a versão é calculada

1. Procura no log (a partir do `HEAD`) o commit **mais recente** cujo assunto
   seja exatamente `release: X.Y.Z`. Se não houver, a base é `0.0.0`.
2. Percorre os commits **depois** dele em **ordem cronológica** e aplica um
   bump por commit, em sequência:

| Commit | Bump | Efeito |
|--------|------|--------|
| `tipo!:` (qualquer tipo) ou `BREAKING CHANGE:` no corpo | major | `X+1.0.0` (**agente não pode gerar** — ver *Breaking change*) |
| `feat` | minor | `X.Y+1.0` (**zera o patch**) |
| `fix` | patch | `X.Y.Z+1` |
| `docs`, `style`, `refactor`, `perf`, `test`, `chore`, `ci` | nenhum | — |
| Assunto fora do padrão `tipo(escopo)?: descrição` | nenhum | — |

Como é sequencial, `feat, feat, feat, fix, feat` a partir de `0.0.0` termina
em `0.4.0`: o `fix` foi "coberto" pelo `feat` seguinte.

Detalhes que mudam o resultado:

- O padrão do assunto é `^(feat|fix|docs|style|refactor|perf|test|chore|ci)(\(escopo\))?(!)?:\s+descrição`.
  Tipo desconhecido ou sem espaço após `:` **não conta**.
- **`BREAKING CHANGE:` só é detectado na primeira linha do corpo** do commit
  (a leitura do `git log` descarta as linhas seguintes). Ao escrever um
  commit **autorizado** como breaking, use `!` no assunto (`feat!: ...`) para
  garantir o major. Agente sem autorização nunca escreve `!` nem essa string.
- Rodapés como `close: #12 - ...` e `Co-Authored-By` não interferem.
- Commits `release: X.Y.Z` fecham o ciclo: o cálculo recomeça deles.

### O que o `commit` altera (somente na RAIZ do repositório)

| Arquivo na raiz | Alteração |
|-----------------|-----------|
| `package.json` | campo `"version"` |
| `pubspec.yaml` | `version: X.Y.Z` e, se houver `+N`, **incrementa o build** (`+N` → `+N+1`); atualiza também `msix_version` se existir |
| `pyproject.toml` | linhas `version = "X.Y.Z"` (ver armadilhas em *Python*) |
| `go.mod` | **não toca o `go.mod`**: cria/sobrescreve o arquivo `VERSION` (ver *Go*) |

- **Só a raiz**: manifestos em subpastas (`api/pubspec.yaml`,
  `packages/*/pubspec.yaml`, `apps/*/pubspec.yaml`) **não são atualizados**.
  Em monorepo, a versão oficial é a do manifesto da raiz.
- Se existirem vários manifestos na raiz (ex.: `package.json` e
  `pubspec.yaml`), todos são atualizados com a mesma versão.
- O `commit` faz `git add` só dos manifestos alterados e depois
  `git commit -m "release: X.Y.Z"` — **qualquer coisa que já esteja no
  stage entra no commit de release**. Antes de rodar, confirme que o índice
  está limpo (`git status`).

### Particularidades por ecossistema

#### Node (`package.json`)

- Troca toda linha que **comece** com `"version": "X.Y.Z"` (só numérico).
- **Não atualiza `package-lock.json`**: o `version` do próprio pacote no
  lock fica defasado até o próximo `npm install`. Rode `npm install` (ou
  `npm install --package-lock-only`) e commite à parte, se o projeto
  versionar o lock.

#### Python (`pyproject.toml`)

Funciona para o caso comum (PEP 621 `[project]` ou Poetry `[tool.poetry]`
com `version = "1.2.3"` estático), mas com limites que o agente precisa
conhecer:

- A regex é `^\s*version\s*=\s*"X.Y.Z"` e **substitui TODAS as linhas que
  casam, em qualquer tabela do arquivo**. Se outra tabela tiver uma linha
  própria `version = "x.y.z"` (ex.: configuração de ferramenta), ela também
  será alterada. Revise o `git diff` do release.
- Só aceita **aspas duplas** e **três números** (`1.2.3`). `version = '1.2.3'`,
  `1.2.3rc1`, `1.2` e versões com sufixo **não casam**.
- **Versão dinâmica** (`dynamic = ["version"]`, versão vinda de
  `__version__`, `setup.py`, `setup.cfg`, `hatch`/`setuptools_scm`) **não é
  suportada**: nada casa, o arquivo fica igual e o `versioner commit` falha ao
  commitar sem mudanças. Nesses projetos use versão estática no
  `pyproject.toml` ou não use o `versioner`.
- Só `pyproject.toml` na raiz. Não atualiza `__init__.py` nem lockfile
  (`poetry.lock`/`uv.lock` não guardam a versão do próprio projeto).

#### Go (`go.mod` → arquivo `VERSION`) — a "gambiarra"

`go.mod` não guarda a versão do módulo (o `go 1.xx` é a versão do
compilador), e o ecossistema Go usa **tags Git** (`v1.2.3`) para isso. Como o
`versioner` **não cria tag**, ele improvisa um arquivo de texto na raiz:

- Se existe `go.mod` na raiz, `versioner commit` **sobrescreve `VERSION`** com
  `X.Y.Z
` e o inclui no commit `release: X.Y.Z`. O arquivo é criado se não
  existir. Só a raiz; módulos Go em subpastas não são vistos.
- `VERSION` é **só uma referência interna** (CI, scripts, o binário). Quem
  consome o módulo com `go get`/`go install pkg@vX.Y.Z` depende da **tag**, que
  continua sendo criada **à mão** pelo dono depois do release.
- Ler a versão no próprio programa é feito por `//go:embed`:

```go
import _ "embed"

//go:embed VERSION
var appVersion string   // usar strings.TrimSpace(appVersion)
```

  Consequências:
  1. `VERSION` precisa estar **no mesmo diretório do pacote** que o embute
     (pacote `main` na raiz) e **existir antes do build**; sem ele o
     `go build` falha ("pattern VERSION: no matching files").
  2. O binário só enxerga a nova versão **depois de recompilar**: rode
     `go build` **depois** de `versioner commit`; o executável compilado antes
     continua com o número antigo.
  3. Versão base lida do `VERSION` não é usada no cálculo; a base sempre vem do
     histórico git (`release: X.Y.Z`), nunca do arquivo.
- Build obrigatório do dono para Go: `go build -ldflags="-s -w" -o <nome> .`
  (remove símbolos e debug). Não commite o binário (`*.exe` no `.gitignore`).

### Fluxo recomendado ao agente

1. Quando o dono pedir versão: `git status` (índice limpo) → `versioner calc`
   → mostrar `Base`/`Next` ao dono.
2. Só com confirmação: `versioner commit`.
3. Reportar o commit criado e os arquivos atualizados. **Não criar tag nem
   dar push.**
