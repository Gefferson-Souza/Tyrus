# 2. Estratégia de Transpilação de Async/Await

Data: 2026-02-10
Status: Aceito

## Contexto

TypeScript e Rust possuem modelos de concorrência diferentes.

- **TS:** Single-threaded Event Loop, `Promise<T>`.
- **Rust:** Multi-threaded (potencialmente), `Future<Output=T>`, requer Runtime (Tokio).

Precisamos definir como transformar funções `async` do TS para Rust de forma que sejam compatíveis com o ecossistema `axum`/`tokio`.

## Decisão

Mapearemos `async/await` do TypeScript diretamente para a sintaxe `async/.await` do Rust, utilizando o crate `tokio` como runtime.

### Regras de Mapeamento:

1.  **Assinatura de Função:**
    - TS: `async function foo(): Promise<string>`
    - Rust: `pub async fn foo() -> Result<String, AppError>`
    - _Nota:_ Todas as funções async devem retornar `Result` para propagação de erros (`?`), mesmo que no TS não lancem exceções explicitamente.

2.  **Unwrapping de Promises:**
    - O tipo de retorno `Promise<T>` é "desembrulhado" para `T` (dentro do `Result`).

3.  **Await Expression:**
    - TS: `await foo()`
    - Rust: `foo().await?`
    - _Nota:_ O operador `?` é adicionado automaticamente para tratar erros, assumindo que qualquer Future pode falhar (padrão em I/O).

4.  **Runtime:**
    - O binário gerado dependerá de `#[tokio::main]`.

## Consequências

### Positivas

- Código gerado é altamente idiomático e legível.
- Integração nativa com crates como `reqwest` e `sqlx` que são `async`.
- Performance superior ao Event Loop do Node.js para tarefas I/O bound.

### Negativas

- Obriga o uso de `tokio` (aumenta o tamanho do binário).
- `async` em traits (Rust) ainda é complexo (embora resolvido no Rust 1.75+ com RPITIT, ainda pode ter edge cases).
