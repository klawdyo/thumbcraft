# Ambiente dos testes de 2026-10-19

Máquina e ferramentas em que todos os números de `RESULTS.md` foram medidos. Versões levantadas por comandos de versão em 2026-10-19.

## Máquina

| Item | Valor |
|---|---|
| Sistema | Ubuntu 24.04.5 LTS, kernel Linux 6.18.44, x86_64, glibc 2.39 |
| CPU | Intel Xeon 2,80 GHz, 2 vCPUs compartilhadas com outros agentes (em geral havia outro build rodando) |
| Memória | 8 GB |
| Disco | 252 GB (cerca de 21 GB livres durante a sessão) |
| Fuso | America/Fortaleza (UTC-3) |
| Fontes do sistema | 2269 faces, incluindo Carlito e Caladea (equivalentes métricos de Calibri e Cambria), Liberation, DejaVu, Inter, Noto (incluindo Noto Color Emoji e Noto Kufi Arabic), IPA Gothic e Droid Sans Fallback. Não há Calibri, Arial nem Times New Roman da Microsoft |

## Versões

| Ferramenta | Versão |
|---|---|
| Rust (rustc) | 1.97.0 (2d8144b78 2026-07-07) |
| cargo | 1.97.0 |
| Alvos do rustup | x86_64-unknown-linux-gnu e wasm32-unknown-unknown (instalado para `cargo xtask wasm`) |
| cargo-bloat | 0.12.1 (instalado na sessão fora de `~/.cargo/bin`, em `/tmp/claude-0/bloat`) |
| LibreOffice | 24.2.7.2 (4:24.2.7-0ubuntu0.24.04.6), pacotes Writer, Calc, Impress, Draw e core |
| ImageMagick | 6.9.12-98 Q16 (`convert`, `montage`, `identify`, `compare`) |
| poppler | `pdftoppm` 24.02.0 |
| qpdf | 11.9.0 |
| pandoc | 3.1.3 |
| Node.js | 22.22.0 |
| jq | 1.7 |
| git | 2.43.0 |
| gh | 2.89.0 |
| xz | 5.4.5 |
| UPX | não instalado |
| Geradores de corpus (Python, não versionados) | python-docx 1.2.0, lxml 6.1.3, python-pptx 1.0.2, Pillow 12.3.0, XlsxWriter 3.2.9, openpyxl 3.1.5 |

## Rede

- Saída HTTPS por um proxy de agente com lista de permissão: alcança GitHub (clone e fetch por git funcionam) e os registries de pacotes (crates.io, npm). Outros hosts, e a API `gh api` da organização storytold, devolvem 403. A busca de issues no GitHub foi bloqueada por robots.
- O GitHub do `storytold/craft-fonts` deu 403. As fontes Arimo, Tinos e Cousine do patch do GridCraft vieram do pacote npm `@expo-google-fonts/{arimo,tinos,cousine}` (MIT e OFL-1.1).
- Dependências git do cargo (`craft-libs`) baixam normalmente.
- Erros de TLS e 403 do proxy: ver `/root/.ccr/README.md` na sessão original (nunca desativar a verificação de TLS).

## Condições de medição

- 2 vCPUs compartilhadas: tempos de execução e de build têm ruído e foram medidos uma vez por configuração. Os tempos de build foram inflados quando dois builds rodaram juntos (DeckCraft) ou quando outro agente compilava.
- Compilar com `CARGO_TARGET_DIR` compartilhado entre builds os serializa (travas do cargo) e, como os binários têm o mesmo nome, é preciso copiar o binário original para fora do `target` antes de compilar a versão modificada (foi o caso do `gridcraft-cli` do teste de regressão).
- Cache de disco quente, perfil do LibreOffice já inicializado (a linha de base de planilhas usou `nice -n 10`).
- Um processo por arquivo, com timeout de 30 s (60 s no LibreOffice de planilhas).
- Só Linux x86_64. Nada foi testado em Windows, macOS ou ARM.
- Para repetir as medições, não rode outras cargas pesadas ao mesmo tempo.
