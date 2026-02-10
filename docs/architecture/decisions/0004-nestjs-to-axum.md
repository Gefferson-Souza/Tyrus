# 4. Mapeamento NestJS para Axum

Data: 2026-02-10
Status: Aceito

## Contexto

O framework alvo do projeto é o **NestJS** (TypeScript). Queremos que o código Rust gerado utilize um framework web robusto.
Escolhemos **Axum** (do ecosistema Tokio) por sua performance, ergonomia e compatibilidade com async.

## Decisão

Transformaremos Decorators do NestJS em rotas e extratores do Axum.

### Regras de Mapeamento:

1.  **Controllers:**
    - TS: `@Controller('users') class UsersController`
    - Rust: Uma função `pub fn router() -> Router` que agrupa as rotas.

2.  **Handlers (Métodos):**
    - TS: `@Get(':id') findOne(...)`
    - Rust: `pub async fn find_one(...)`
    - A rota é registrada no Router: `.route("/:id", get(find_one))`

3.  **Extractors (@Body, @Param):**
    - TS: `create(@Body() user: UserDto)`
    - Rust: `create(Json(user): Json<UserDto>)`
    - O argumento é movido para o padrão de Extractor do Axum.

4.  **Injeção de Dependência:**
    - O `Dependency Injection` do NestJS é simulado passando o `State` (Estado da Aplicação) para os handlers.
    - O `Service` é instanciado no `main.rs` e passado via `.with_state(service)`.

## Consequências

### Positivas

- Axum é extremamente rápido.
- O modelo de Extractors do Axum mapeia limpo para Decorators.

### Negativas

- Perda de alguns recursos dinâmicos do NestJS (Middleware complexo, Guards) que precisarão ser reimplementados "à la Rust" (tower middlewares).
