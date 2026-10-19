# Princípios de arquitetura (valem para toda linguagem)

> Fonte: projeto SEMI (o mais recente e o mais fiel ao que o dono quer).
> Se um projeto antigo contradisser este arquivo, **este arquivo vence**.
> A única exceção declarada é o Vue legado (ver `vue/`, quando existir).

## 1. Módulos/features, nunca MVC

Todo projeto é dividido **fisicamente por módulo de negócio (feature)**, e
cada módulo carrega tudo o que é seu dentro da própria pasta.

- **Proibido** organizar por tipo técnico no nível raiz (`controllers/`,
  `models/`, `views/`, `services/` soltos na raiz do projeto).
- Dentro do módulo, aí sim se separa por camada/tipo (ver o guia da
  linguagem: `flutter/structure.md`, `nestjs/structure.md`).
- Um módulo **não importa os internos de outro módulo**. Comunicação entre
  módulos acontece por: interface pública exportada, evento, ou injeção de
  uma abstração. Nunca por importar a classe concreta alheia.
- Código realmente transversal (infraestrutura, tema, rede, storage, utilidades)
  vive em uma pasta de suporte separada (`core/`, `lib/`, `shared/`), nunca
  dentro de um módulo de negócio.

## 2. Classes desacopladas com injeção de dependências

- Nenhuma classe cria suas dependências com `new`/construtor direto,
  nem usa singleton global nem service locator estático.
- Dependências entram **pelo construtor**, tipadas como **abstração**
  (interface/classe abstrata) sempre que houver fronteira com I/O (banco,
  rede, disco, relógio, provedor externo).
- O "fio" concreto (qual implementação vai em qual abstração) é montado em
  **um único lugar** por módulo: Riverpod providers (Dart), módulo Nest
  (TypeScript), `provide/inject` ou store (Vue).
- **Proibido carregar dependência dentro de método** (`await import()`,
  `require()`, `new` de cliente/SDK). Em NestJS ver
  `nestjs/dependency-injection.md`; a regra vale em qualquer stack.
- Se uma classe é difícil de testar porque instancia algo por conta própria,
  o defeito é de **design (SOLID)** — corrija a classe, não contorne com
  mock frágil.

## 3. Regra de negócio fora da visualização e 100% testável

- Telas, widgets, controllers HTTP e componentes **só** apresentam, traduzem
  entrada/saída e delegam. Zero regra de negócio neles.
- Regra de negócio vive em: classes de domínio (invariantes), services,
  use cases e notifiers/stores — todos testáveis sem framework de UI e sem
  infraestrutura real.
- Teste unitário **nunca** sobe banco, rede ou disco reais. Usa fakes/mocks
  das abstrações injetadas.
- **Única divergência aceita:** o frontend Vue legado, onde parte da lógica
  ainda está nos componentes/stores. Em código **novo**, mesmo em Vue, siga a
  regra; em código legado, não reescreva por conta própria (ver `vue/`).

## 4. Camadas (arquitetura hexagonal / clean, SOLID)

Dependência sempre aponta **para dentro**, em direção ao domínio:

```
presentation / controller  →  application (use case, service, notifier)
                           →  domain (entidades, regras, ports/interfaces)
data / infrastructure      →  implementa os ports do domain
```

- **domain**: entidades, value objects, exceções de domínio e as interfaces
  (ports) que ele precisa. Sem framework, sem ORM, sem HTTP, sem Flutter.
- **application**: orquestra casos de uso; único lugar que muta estado e
  chama repositórios/serviços.
- **data / infrastructure**: adaptadores (TypeORM, HTTP, storage) que
  implementam os ports.
- **presentation**: telas/controllers; leem estado e encaminham intenção.

Se um service/notifier acumula mais de uma responsabilidade, **extraia um
use case dedicado e injetável**. Sinal típico: o teste precisa de setup
enorme ou cobre cenários demais.

## 5. Entidades, domínio e interfaces

- Classes de domínio **nunca aceitam estado inválido**: validam no
  construtor/setter e lançam exceção de domínio tipada (ver
  `shared/errors-and-contracts.md`).
- Preferir imutabilidade onde a linguagem facilita (Dart: `final` +
  `copyWith`). Em TypeScript, getters/setters com validação.
- Entidade de persistência (ORM) e classe de domínio implementam a **mesma
  interface** do contrato (`IProduct`), para que mudanças de campo quebrem a
  compilação nos dois lados.
- DTO é formato de transporte (wire). Nunca vaza do `data`/controller para
  o domínio ou para a UI: converte-se por mapper/factory.

## 6. Arquivos pequenos e reuso por parâmetro

- Se dois componentes/widgets diferem por um parâmetro, **é um só**,
  parametrizado.
- Nomes de componentes compartilhados são **agnósticos de módulo**
  (nunca `SalesX` para algo usado por compras e vendas).
- Variantes e escolhas restritas são **enum**, nunca `String` solta.

## 7. Contrato entre front e back é sagrado

Mudança em entidade, DTO, rota, enum compartilhado, envelope de erro, auth
ou permissão **primeiro** é documentada/aprovada no lugar do contrato e só
depois implementada, nos dois lados. Ver `shared/api-contract.md` e
`shared/workflow-and-agents.md`.

## 8. Proibido inventar padrão novo

Antes de criar uma estrutura nova, procure o padrão existente no projeto e
neste repositório de specs. Só diverja justificando por que o existente não
serve — e registre a decisão.
