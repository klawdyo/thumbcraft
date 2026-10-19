# GridCraft: gp-min com papel A4 (comparação A3 contra A4)

> Arquivo de relatório. Origem: subagente `a0d9c86c9a242e39e`, turno 2 (de 3). Sessão de 2026-10-19.
> Tema: Ajuste do usuário: A4 em vez de A3.
> Conteúdo fiel ao original. Alterações do arquivista: convenção sem travessões, notas de correção de unidade entre colchetes e o prefixo de caminho da sessão (`/tmp/claude-0/.../scratchpad`) abreviado para `S`. Esses caminhos não existem mais: veja `reports/README.md` para saber onde cada artefato está neste repositório.

---

A4 virou o padrão no `gp-min`, e o resultado no corpus é o mesmo de antes: 54 de 57 arquivos geraram PNG. As 3 falhas continuam sendo os arquivos corrompidos (`49`, `50`, `51`), que falham na abertura. O MANIFEST.tsv ficou de fora, como pedido.

**Binário:** 11.409.736 bytes (10,88 MiB), idêntico ao de A3, porque a única mudança é a string do papel. O rebuild release com as mesmas otimizações levou 2 min 27 s.

**Tempos no corpus A4:** a soma foi 6,06 s (6,8 s de relógio no script) e o maior arquivo levou 456 ms. Em A3 o maior tinha sido 301 ms. Não vi de qual arquivo veio o 456 ms nem se é variação de carga, já que o outro agente também compila no mesmo target.

**A3 contra A4 (pixels a escala 2):**

| Caso | A3 | A4 |
|---|---|---|
| Planilha larga, 130 colunas (recorte A1:Z60) | 2318x847 | 1632x600 |
| 10 mil linhas | 232x1804 | 213x1645 |
| Painéis congelados | 1440x1804 | 1151x1440 |
| Gráficos, texto longo, `01-texto-acentos` | iguais | iguais |

- **Quando muda:** a página só altera o resultado quando o conteúdo não cabe nela. O motor reduz a escala e nunca amplia acima de 100%. Em A4 a planilha larga fica cerca de 30% menor.
- **Legibilidade:** a planilha larga em A4 ainda dá para ler a 1632 px, mas o texto é bem menor. Em miniatura, A3 lê melhor.
- **Texto cortado na célula:** não melhora nem piora. "Centro-O", "Olá, cora" e "Transbordo de texto: este t" saem idênticos em A3 e A4. O corte é regra do motor, não da página.

**Escala maior:** compensa para planilhas largas. A planilha de 130 colunas em A4 sai com 1632, 2439 e 3246 px de largura nas escalas 2, 3 e 4. A4 com escala 3 (2439 px) dá texto de tamanho parecido ao A3 com escala 2 (2318 px). Não medi o tempo nem o tamanho de arquivo com `--scale 3`.

**Padrão que recomendo:** A4 com `--scale 2`, para thumbnails comuns. Para planilhas largas ou longas, que o motor encolhe, usar `--scale 3`. O motor não informa o fator de redução, então o programa teria que calcular isso. Hoje a escolha é manual.

A imagem lado a lado tem 6 casos (A3 com borda vermelha, A4 com borda azul). Os dois últimos casos (10 mil linhas e painéis congelados) ficaram sem legenda e com o texto pequeno demais para ler. Os 4 anteriores estão bons.

Caminhos:
- `S/grid-png/b-pdfraster/out-a4/` (PNGs e `results.tsv`)
- `S/grid-png/b-pdfraster/compare-a3-a4.png`
- Binário A4: `S/grid-png/target/release/gp-min`
- Cópia do binário A3, para comparação: `S/grid-png/b-pdfraster/gp-min-a3.bin`
- `b-pdfraster/out/` (A3) ficou intacto e o GridCraft original não foi tocado.
