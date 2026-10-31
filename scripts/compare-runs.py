#!/usr/bin/env python3
"""Compara duas rodadas do scripts/run-corpus.sh (antes e depois de uma atualização dos *craft).

Uso: scripts/compare-runs.py ANTES DEPOIS [--out DIR] [--top N] [--limiar L]

ANTES e DEPOIS são pastas de saída do run-corpus.sh (PNG + results.tsv). Para cada arquivo do
corpus o script classifica a mudança e mede a diferença de pixels. Saída em DIR (padrão: ./comparacao):
  comparacao.tsv   uma linha por arquivo (classe, códigos, dimensões, % de pixels diferentes, diferença média)
  resumo.md        contagem por classe e a lista dos casos que pedem olhar
  pares/*.png      antes | depois | diferença dos N casos mais diferentes

Classes: IGUAL (bytes idênticos), PIXEL_IGUAL (PNG diferente, pixels iguais), PEQUENA, MEDIA, GRANDE
(pela fração de pixels que diferem mais que o limiar), DIMENSAO (tamanho da imagem mudou),
NOVO_ERRO (saía 0 e agora falha), CORRIGIDO (falhava e agora sai 0), ERRO_IGUAL, CODIGO (outro código
de erro). Códigos de saída: 0 se nenhum NOVO_ERRO, 1 se houver.
"""
import argparse
import csv
import sys
from pathlib import Path

import numpy as np
from PIL import Image


def ler_tsv(pasta: Path) -> dict:
    arq = pasta / "results.tsv"
    if not arq.is_file():
        sys.exit(f"erro: {arq} não existe (rode scripts/run-corpus.sh)")
    with arq.open(encoding="utf-8", newline="") as f:
        return {r["arquivo"]: r for r in csv.DictReader(f, delimiter="\t")}


def png_de(pasta: Path, rel: str) -> Path:
    return pasta / (rel.replace("/", "_") + ".png")


def carregar(caminho: Path) -> np.ndarray:
    """RGBA composto sobre branco, como RGB uint8."""
    im = Image.open(caminho).convert("RGBA")
    fundo = Image.new("RGBA", im.size, (255, 255, 255, 255))
    return np.asarray(Image.alpha_composite(fundo, im).convert("RGB"))


def alinhar(a: np.ndarray, b: np.ndarray):
    h, w = max(a.shape[0], b.shape[0]), max(a.shape[1], b.shape[1])

    def pad(x):
        out = np.full((h, w, 3), 255, np.uint8)
        out[: x.shape[0], : x.shape[1]] = x
        return out

    return pad(a), pad(b)


def classe_pixels(pct: float) -> str:
    if pct == 0:
        return "PIXEL_IGUAL"
    if pct < 0.5:
        return "PEQUENA"
    if pct < 5:
        return "MEDIA"
    return "GRANDE"


def par(a: np.ndarray, b: np.ndarray, d: np.ndarray, destino: Path, altura: int = 420):
    mapa = np.full(a.shape, 255, np.uint8)
    mapa[d > 0] = (230, 30, 30)
    tiras = []
    for x in (a, b, mapa):
        im = Image.fromarray(x)
        k = altura / im.height
        tiras.append(im.resize((max(1, round(im.width * k)), altura), Image.LANCZOS))
    folha = Image.new("RGB", (sum(t.width for t in tiras) + 20, altura), (120, 120, 120))
    x0 = 0
    for t in tiras:
        folha.paste(t, (x0, 0))
        x0 += t.width + 10
    folha.save(destino)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("antes", type=Path)
    ap.add_argument("depois", type=Path)
    ap.add_argument("--out", type=Path, default=Path("comparacao"))
    ap.add_argument("--top", type=int, default=25)
    ap.add_argument("--limiar", type=int, default=16, help="diferença por canal (0-255) que conta como pixel diferente")
    args = ap.parse_args()

    ta, tb = ler_tsv(args.antes), ler_tsv(args.depois)
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "pares").mkdir(exist_ok=True)
    linhas = []
    for rel in sorted(set(ta) | set(tb)):
        ra, rb = ta.get(rel), tb.get(rel)
        if ra is None or rb is None:
            linhas.append({"arquivo": rel, "classe": "SO_EM_UM", "motor": (ra or rb)["motor"]})
            continue
        ca, cb = ra["saida"], rb["saida"]
        l = {"arquivo": rel, "motor": rb["motor"], "saida_antes": ca, "saida_depois": cb,
             "dim_antes": ra["dimensoes"], "dim_depois": rb["dimensoes"],
             "ms_antes": ra["ms"], "ms_depois": rb["ms"], "pct_pixels": "", "dif_media": ""}
        if ca == "0" and cb != "0":
            l["classe"] = "NOVO_ERRO"
        elif ca != "0" and cb == "0":
            l["classe"] = "CORRIGIDO"
        elif ca != "0":
            l["classe"] = "ERRO_IGUAL" if ca == cb else "CODIGO"
        else:
            pa, pb = png_de(args.antes, rel), png_de(args.depois, rel)
            if pa.read_bytes() == pb.read_bytes():
                l["classe"] = "IGUAL"
                l["pct_pixels"], l["dif_media"] = "0.000", "0.000"
            else:
                a, b = carregar(pa), carregar(pb)
                mudou = a.shape != b.shape
                a, b = alinhar(a, b)
                d = np.abs(a.astype(np.int16) - b.astype(np.int16)).max(axis=2)
                pct = float((d > args.limiar).mean() * 100)
                l["pct_pixels"], l["dif_media"] = f"{pct:.3f}", f"{float(d.mean()):.3f}"
                l["classe"] = "DIMENSAO" if mudou else classe_pixels(pct)
                l["_par"] = (a, b, np.where(d > args.limiar, d, 0))
        linhas.append(l)

    campos = ["arquivo", "motor", "classe", "saida_antes", "saida_depois", "dim_antes", "dim_depois",
              "pct_pixels", "dif_media", "ms_antes", "ms_depois"]
    with (args.out / "comparacao.tsv").open("w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=campos, delimiter="\t", extrasaction="ignore", restval="")
        w.writeheader()
        w.writerows(linhas)

    por_classe = {}
    for l in linhas:
        por_classe.setdefault(l["classe"], []).append(l)
    mais = sorted((l for l in linhas if "_par" in l), key=lambda l: -float(l["pct_pixels"]))[: args.top]
    for l in mais:
        nome = l["arquivo"].replace("/", "_") + ".png"
        par(*l["_par"], args.out / "pares" / nome)

    ordem = ["NOVO_ERRO", "CODIGO", "DIMENSAO", "GRANDE", "MEDIA", "PEQUENA", "PIXEL_IGUAL", "IGUAL", "CORRIGIDO",
             "ERRO_IGUAL", "SO_EM_UM"]
    res = [f"# Comparação {args.antes} contra {args.depois}", "", f"{len(linhas)} arquivos; limiar {args.limiar}/255 por canal.", "",
           "| Classe | Arquivos |", "|---|---|"]
    res += [f"| {c} | {len(por_classe[c])} |" for c in ordem if c in por_classe]
    for c in ("NOVO_ERRO", "CODIGO", "DIMENSAO", "GRANDE", "MEDIA"):
        if c in por_classe:
            res += ["", f"## {c}", "", "| Arquivo | Motor | Antes | Depois | % pixels | Dif. média |", "|---|---|---|---|---|---|"]
            for l in sorted(por_classe[c], key=lambda l: -float(l["pct_pixels"] or 0)):
                antes = l["saida_antes"] if c in ("NOVO_ERRO", "CODIGO") else l["dim_antes"]
                depois = l["saida_depois"] if c in ("NOVO_ERRO", "CODIGO") else l["dim_depois"]
                res.append(f"| {l['arquivo']} | {l['motor']} | {antes} | {depois} | {l['pct_pixels']} | {l['dif_media']} |")
    (args.out / "resumo.md").write_text("\n".join(res) + "\n", encoding="utf-8")
    print("\n".join(res[:3 + 2 + len(por_classe) + 1]))
    return 1 if "NOVO_ERRO" in por_classe else 0


if __name__ == "__main__":
    sys.exit(main())
