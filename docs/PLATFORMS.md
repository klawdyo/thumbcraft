# Plataformas suportadas: build e testes

Plataformas: Windows x64 e Linux x64 (glibc). macOS, ARM e musl ficam de fora (tarefa `.tasks/05-platform-builds.md`).

| Plataforma | Rust | `cargo test --workspace --locked` | Binário de release | Data |
|---|---|---|---|---|
| Windows 11 x64 | 1.99.1 | 21 + 26 (e2e) + 81 + 10 + 2 = 140 testes, verdes | 25.669.632 bytes (24,48 MiB) | 2026-10-19 |
| Linux x64 glibc | | pendente nesta versão do código | | |

Referências anteriores: Windows sem PDF, 23.995.392 bytes (2026-10-19); com PDF e craft-libs duplicado, 26.701.824;
sem o krilla, 25.655.296. Linux, avaliação de 2026-10-19 (código anterior ao PDF): `baselines/` e `RESULTS.md`.

## Como reproduzir

```bash
scripts/fetch-upstream.sh
cargo test --workspace --locked
cargo build --release --locked -p thumbcraft
```

O `fetch-upstream.sh` fixa `core.autocrlf=false` e confere o hash da árvore do GridCraft (o bit de execução vem do
commit; ver `PITFALLS.md`). No Windows, o Acesso Controlado a Pastas do Defender pode bloquear o build dentro do
projeto até cada executável ser aprovado.
