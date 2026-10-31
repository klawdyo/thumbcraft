//! Uma versão por pacote no `Cargo.lock` (CLAUDE.md, "Zonas protegidas"; docs/UPSTREAM-CHANGES.md).
//!
//! O objetivo do projeto é um binário pequeno e padronizado: duas versões do mesmo crate
//! (por exemplo `skrifa` ou `read-fonts`) dobram código no binário. Uma atualização dos *craft costuma
//! trazer isso de volta (um motor passa a pedir `miniz_oxide` 0.9, outro `skrifa` 0.47, e assim por
//! diante); este teste falha na hora e aponta o pacote. A correção é um patch pequeno em `patches/`
//! que alinha a versão, registrado em docs/UPSTREAM-CHANGES.md.

use std::collections::BTreeMap;
use std::path::Path;

/// Duplicatas aceitas, cada uma com o motivo. Antes de acrescentar uma, confirme com
/// `cargo tree -p thumbcraft -d -e normal,build --target all` que o pacote não entra no binário.
const ACEITAS: &[(&str, &str)] = &[("syn", "só proc-macros (tempo de compilação): syn 2 em phf_macros e zerocopy-derive, syn 3 nos demais")];

/// Pacotes que os três motores compartilham: se aparecerem duas vezes, a união deixou de funcionar.
const COMPARTILHADOS: &[&str] = &[
    "skrifa", "read-fonts", "font-types", "harfrust", "vello_cpu", "vello_common", "glifo", "kurbo", "peniko", "png", "zip", "quick-xml",
    "image", "miniz_oxide", "flate2", "craft-pdf", "hayro", "hayro-interpret", "hayro-syntax",
];

fn versoes() -> BTreeMap<String, Vec<String>> {
    let lock = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.lock");
    let texto = std::fs::read_to_string(&lock).unwrap_or_else(|e| panic!("sem {}: {e}", lock.display()));
    let mut mapa: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut nome: Option<String> = None;
    for linha in texto.lines() {
        if let Some(n) = linha.strip_prefix("name = \"").and_then(|r| r.strip_suffix('"')) {
            nome = Some(n.to_string());
        } else if let (Some(v), Some(n)) = (linha.strip_prefix("version = \"").and_then(|r| r.strip_suffix('"')), nome.take()) {
            mapa.entry(n).or_default().push(v.to_string());
        }
    }
    mapa
}

#[test]
fn lock_has_one_version_per_package() {
    let duplicados: Vec<String> = versoes()
        .into_iter()
        .filter(|(n, v)| v.len() > 1 && !ACEITAS.iter().any(|(a, _)| a == n))
        .map(|(n, v)| format!("{n} {}", v.join(", ")))
        .collect();
    assert!(
        duplicados.is_empty(),
        "pacotes com mais de uma versão no Cargo.lock: {duplicados:?}. Alinhe por patch (docs/UPSTREAM-CHANGES.md, \
         patches 1 a 5) ou, se o pacote não entra no binário, registre em ACEITAS com o motivo"
    );
}

#[test]
fn shared_packages_are_present_once() {
    let mapa = versoes();
    for p in COMPARTILHADOS {
        let v = mapa.get(*p).unwrap_or_else(|| panic!("{p} sumiu do Cargo.lock: atualize COMPARTILHADOS se foi de propósito"));
        assert_eq!(v.len(), 1, "{p} aparece em {} versões: {v:?}", v.len());
    }
}

#[test]
fn accepted_duplicates_are_still_needed() {
    let mapa = versoes();
    for (p, _) in ACEITAS {
        let n = mapa.get(*p).map_or(0, Vec::len);
        assert!(n > 1, "{p} não está mais duplicado: tire-o de ACEITAS");
    }
}
