# Política de testes

> **TDD é obrigatório. Teste antes da implementação, sempre.**

## Ciclo

1. **Red**: escrever o teste contra os critérios de aceitação; confirmar que
   falha **pelo motivo certo** (implementação ausente, não erro de setup).
2. **Green**: implementar o mínimo até passar.
3. **Refactor**: limpar sem mudar comportamento.

## Regras comuns

- **Nunca** subir infraestrutura real (banco, rede, disco, relógio) em teste
  unitário. Toda dependência entra por injeção e é substituída por fake/mock.
- Fakes reutilizáveis ficam em pasta própria (`test/fakes/`, `src/**/fakes`)
  e são compartilhados; não copie o mesmo mock em vários testes.
- Teste cobre **comportamento observável descrito nos critérios de
  aceitação**. Não trave refatorações que não mudam comportamento.
- **Nunca** inventar comportamento não descrito nos critérios.
- **Nunca** relaxar um critério para o teste passar.
- Critério vago demais para virar teste objetivo → a tarefa volta a quem a
  escreveu (CTO), não se improvisa.
- Se a classe é difícil de testar isoladamente, é problema de SOLID/DI:
  reportar e corrigir o design.
- Se o service/notifier exige setup gigante, **extrair use cases**.
- Commits de teste: tipo `test:`, atômicos, um por peça de implementação.

## O que testar em cada camada

| Camada | Testa |
|--------|-------|
| Domínio | Invariantes (estados inválidos lançam exceção de domínio), cálculos puros |
| Application (service/use case/notifier/store) | Regras de negócio, orquestração, tratamento de erro — com fakes das portas |
| Data/adapters | Mapeamento DTO↔domínio, montagem de requisição, interpretação de erro HTTP |
| Presentation | Estrutura e comportamento renderizado/roteado. **Nunca** regra de negócio: se o teste de UI precisa exercer regra, a lógica vazou — corrija a arquitetura |

## Específico por stack

- Dart/Flutter: ver `flutter/testing.md`.
- NestJS: ver `nestjs/testing.md`.
