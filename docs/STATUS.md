# Status da sessão (2026-10-19, 10:25)

Os trabalhos foram interrompidos por falta de tokens e serão retomados às 11:21 (America/Sao_Paulo).

## Frentes
1. Construtor do binário unificado: interrompido no início (código de trim, resize, roteamento e trait de render com fake). Estado do que existe: ver `git status` e `crates/`.
2. Arquivista (corpora, baselines, relatórios, docs, README): interrompido no início, quase nada gravado.
3. Benchmark de PDF (pdf-parse contra craft-pdf, pdftoppm, mutool): interrompido nos helpers.

## Pendências
- Push recusado: falta instalar o app GitHub do Claude na organização.
- Retomar as três frentes a partir do material em disco, sem refazer o que já existe.
- Dúvida do usuário sobre craft-libs ser "core": ver resposta no chat (é dependência compartilhada, mas cada projeto pina uma revisão diferente e o raster de PDF só existe no main).

## Decisão (10:26): benchmarks fora do projeto
O código dos benchmarks (TypeScript, pdf-min, scripts) NÃO fica no repositório. Entram apenas o método (`docs/PDF.md`) e os relatórios de resultado. O código de trabalho está em `$SCRATCHPAD/pdf-bench-work` (fora do git). PDFs grandes também ficam só como receita.

## Pendência (10:27): CI/CD de release
Depois que os tokens voltarem (a partir das 11:21) e o binário unificado estiver funcionando: criar o CI/CD (GitHub Actions) que gera o binário de release para Windows e Ubuntu 24. Pontos a resolver: fetch dos upstream em SHAs fixos e aplicação do patch do GridCraft no workflow, cache do cargo, perfil de release para tamanho, publicação dos artefatos em uma release. Não fazer antes.

## Princípio: repositório com código puro
O projeto guarda código, patches, documentação, método e relatórios de resultado. Sem benchmarks, binários, node_modules, target/ ou arquivos grandes.
