> 🌐 **Idioma:** [English](README.md) | Português (BR)

# Tyrus: Um Compilador TypeScript-para-Rust de Alta Fidelidade

_Projeto experimental em teoria de compiladores e preservação semântica_

[![CI Status](https://github.com/Gefferson-Souza/Tyrus/actions/workflows/ci.yml/badge.svg)](https://github.com/Gefferson-Souza/Tyrus/actions/workflows/ci.yml)
![Rust Version](https://img.shields.io/badge/rust-1.75%2B-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

Tyrus é um compilador source-to-source projetado para conectar a sintaxe dinâmica de alto nível (TypeScript) à segurança de memória de baixo nível (Rust). Concentra-se no mapeamento formal de abstrações de ordem superior para equivalentes Rust de custo zero, explorando os limites da **Preservação Semântica** entre modelos de execução distintos.

**Marco:** O Tyrus agora compila uma API NestJS CRUD em um servidor Axum (Rust) funcional:

```bash
tyrus compile meu-projeto-nestjs/src --output servidor-rust
# → Binário Rust com endpoints GET/POST, DI e estado em memória
```

---

## 🔬 Filosofias Fundamentais

### 🛡️ Zero Magia

Tyrus não depende de um runtime oculto ou emulação de garbage collection. Cada construção TypeScript é mapeada para seu equivalente Rust mais eficiente, aproveitando o modelo de ownership e o sistema de tipos estrito do Rust em tempo de compilação.

### 📐 Preservação Semântica

O objetivo principal é a equivalência formal. Se um projeto TypeScript for "Oxidizável", o código Rust gerado deve manter o comportamento da lógica original. Isso ainda não é garantido: os padrões listados abaixo são os verificados até agora.

### 🚫 O Padrão Oxidizável

Tyrus impõe um subconjunto estrito de TypeScript chamado "Padrão Oxidizável". Ele rejeita padrões não idiomáticos ou inseguros (como `any` ou `eval`) para garantir que o código Rust resultante seja seguro e performático.

### 🔐 Arquitetura de Transpilação Segura

Aderindo aos princípios estritos de "Transpilação Segura":

- **Compilação Livre de Panic**: A lógica do compilador utiliza tratamento robusto de erros em vez de entrar em panic com entrada inválida.
- **Linting Estrito**: O código-fonte é verificado com regras `clippy::pedantic` (por exemplo, sem `unwrap()`/`expect()` em caminhos de produção).
- **Mapeamento Formal de AST**: Utiliza Tipos de Dados Algébricos (ADTs) para representar a lógica, evitando vulnerabilidades de manipulação de strings.

---

## 🚀 Camadas de Funcionalidades

### Camada 1: Linguagem Base (Experimental)

- Primitivos (`string`, `number`, `boolean`)
- Fluxo de Controle (`if/else`, `while`, `for`)
- Tratamento de Erros (`Result`, `Option`)

### Camada 2: Sistema de Tipos Avançado (Experimental)

- Interfaces e Type Aliases para Structs/Enums
- Generics e Polimorfismo
- Mapeamento Abrangente de Coleções (`Array<T>` -> `Vec<T>`)

### Camada 3: Ecossistema e Assincronia (Experimental)

- `Async/Await` para concorrência baseada em Future
- Serialização/Desserialização JSON (via `serde`)
- Cliente HTTP e padrões REST (via `axum` & `reqwest`)

### 📦 Padrões Suportados (Verificados com Testes de Equivalência Semântica)

- **Métodos de String** (16): `includes`, `replace`, `split`, `toUpperCase`, `toLowerCase`, `trim`, `startsWith`, `endsWith`, `toString`, `substring`, `charAt`, `indexOf`, `repeat`, `slice`, `padStart`, `padEnd`
- **Métodos de Array** (15): `map`, `filter`, `forEach`, `find`, `some`, `every`, `reduce`/`fold`, `join`, `includes`, `push`, `indexOf`, `slice`, `concat`, `reverse`, `pop`, `sort`, `shift`, `flat`, `flatMap`
- **Funções Math** (15): `max`, `min`, `round`, `floor`, `ceil`, `abs`, `random`, `pow`, `sqrt`, `log`, `trunc`, `sign`, `sin`, `cos`, `tan`
- **Constantes Math** (2): `Math.PI`, `Math.E`
- **Console** (5): `log`, `error`, `warn`, `info`, `debug`
- **Fluxo de Controle**: `if/else`, `while`, `for`, `for-of`, `do-while`, `switch/case`, ternário, `try/catch`
- **Top-Level Statements**: `const`, `let`, expressões auto-wrapped em `fn main()`
- **Spread Operator**: `[...arr1, ...arr2]` → iterator chain, `{...obj, field}` → struct update
- **Herança de Classe**: `extends`, `super()`, override de métodos via field flattening
- **Getters/Setters**: `get prop()` → `fn prop(&self)`, `set prop(v)` → `fn set_prop(&mut self, v)`
- **Métodos Estáticos**: `static add()` → função associada, `Class.method()` → `Class::method()`
- **Type Assertions**: `as Type` → no-op (apenas compile-time)
- **Enums Numéricos**: `enum Direction { Up = 0 }` → `#[repr(i32)] enum` com `Display`
- **Operadores**: Aritméticos, comparação, lógicos, `**` (exponenciação), `%` (módulo)
- **Estado de Classe**: Encapsulamento automático com `Arc<Mutex<T>>` para services/controllers
- **Interfaces**: `interface` -> `#[derive(Serialize, Deserialize)] struct`
- **Unions de String**: `type Status = "a" | "b"` -> `enum` com `Display` e `PartialEq`
- **Guards NestJS**: `@UseGuards(Guard)` → `axum::middleware::from_fn()` layer

---

## 🛠 Instalação e Uso

### Pré-requisitos

- Rust 1.75+ (Stable)
- Cargo

### Configuração

```bash
git clone https://github.com/Gefferson-Souza/Tyrus.git
cd Tyrus
cargo build --release
```

### Instalar Globalmente

```bash
cargo install --path crates/tyrus_cli
tyrus --version
```

### Usando o Compilador

```bash
# Analisar um arquivo TypeScript para compatibilidade
tyrus check ./src/index.ts

# Diagnósticos JSON (para integração com ferramentas)
tyrus check --json ./src/index.ts

# Transpilar para código Rust (stdout ou arquivo)
tyrus build ./src/index.ts
tyrus build ./src/index.ts -o output.rs

# Transpilar + compilar para binário nativo
tyrus compile ./src/index.ts --output ./output

# Transpilar + compilar + executar
tyrus run ./src/index.ts --output ./output

# Suprimir banner para scripting
tyrus --quiet check ./src/index.ts
```

---

## 📋 Referência de Comandos

**Comandos de Desenvolvimento:**

<!-- AUTO-GENERATED from Cargo.toml and CI -->
| Comando | Descrição |
|---------|-----------|
| `cargo build --workspace` | Compilar todas as crates do workspace |
| `cargo build --release` | Build de produção com LTO |
| `cargo install --path crates/tyrus_cli` | Instalar CLI `tyrus` globalmente |
| `cargo nextest run --workspace` | Executar todos os testes (paralelo, recomendado) |
| `cargo test --workspace` | Executar todos os testes (runner legado) |
| `cargo test -p integration_tests` | Apenas testes de integração |
| `cargo clippy --workspace` | Lint com regras estritas (`-Dwarnings` aplicado) |
| `cargo fmt -- --check` | Verificar formatação |
| `cargo insta review` | Revisar mudanças em snapshots |

**Comandos CLI do Tyrus (após instalação global):**

| Comando | Descrição |
|---------|-----------|
| `tyrus check <file.ts>` | Analisar compatibilidade Oxidizável |
| `tyrus check --json <file.ts>` | Diagnósticos em JSON |
| `tyrus build <file.ts>` | Transpilar para Rust (stdout) |
| `tyrus build <file.ts> -o output.rs` | Transpilar para Rust (arquivo) |
| `tyrus compile <file.ts> -o <dir>` | Transpilar + compilar para binário nativo |
| `tyrus run <file.ts>` | Transpilar + compilar + executar |
| `tyrus --quiet <command>` | Suprimir banner para scripting |
<!-- /AUTO-GENERATED -->

---

## 🧪 Suite de Testes

195 testes distribuídos em 8 tipos de teste e 4 camadas de funcionalidades:

| Tipo | Quantidade | Descrição |
|------|-----------|-----------|
| **Equivalência** | 81 | Prova semântica: TS e Rust produzem stdout idêntico |
| **Unitário** | 49 | Rápido, funções isoladas de codegen |
| **Snapshot** | 20 | Saída completa de transpilação via `insta` |
| **IR** | 21 | Lowering de representação intermediária tipada |
| **Compilação** | 9 | Rust gerado passa no `cargo check` por camada |
| **CLI** | 7 | Testes de integração para todos os comandos e flags |
| **Trybuild** | 1 | Verificação de compilação do Rust gerado |
| **E2E/Build** | 5 | Pipeline completo: transpilar → compilar → executar servidor → verificar HTTP |

Tipos de teste: **Equivalência** (TS↔Rust mesma saída) · **Unitário** (funções rápidas e isoladas) · **Snapshot** (insta, saída do codegen) · **IR** (type lowering) · **Compilação** (Rust gerado passa no `cargo check`) · **CLI** (integração de comandos) · **E2E** (verificação de servidor HTTP) · **Trybuild** (verificação de compilação)

---

## 📖 Tese e Arquitetura

Para um aprofundamento nos internos do compilador, consulte [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

---

## 🏗 Estrutura dos Módulos de Geração de Código

A crate `tyrus_codegen` é organizada em módulos focados e com responsabilidade única em `crates/tyrus_codegen/src/convert/`:

```
convert/
├── mod.rs          — declarações de módulo e re-exports
├── interface.rs    — definição da struct RustGenerator + impl Visit (ponto de entrada)
├── helpers.rs      — utilitários compartilhados: to_snake_case, to_pascal_case, is_string_expr
├── fn_decl.rs      — processamento de declaração de funções (process_fn_decl)
├── module.rs       — manipulação de módulos/imports
├── type_mapper.rs  — mapeamento de tipos TypeScript → Rust (map_type_core deduplicado)
├── stmt/           — conversão de statements
│   ├── mod.rs          — dispatcher + convert_stmt, convert_stmt_recursive
│   ├── var_decl.rs     — declarações de variáveis (ident, desestruturação objeto/array)
│   ├── loops.rs        — while, for-of, for-in, for, do-while
│   ├── switch.rs       — switch → match
│   └── try_catch.rs    — try-catch → Result matching
├── class/          — class → struct+impl
│   ├── mod.rs          — dispatcher + conversão de propriedades
│   ├── constructor.rs  — transpilação de construtores + DI
│   ├── method.rs       — transpilação de métodos + decorators
│   ├── routing.rs      — geração de router Axum + middleware @UseGuards
│   └── mutation.rs     — detecção de self-mutation
└── expr/
    ├── mod.rs      — dispatcher de expressões (convert_expr)
    ├── binary.rs   — operadores binários (convert_bin_expr)
    ├── call.rs     — chamadas de função/método, métodos axios/fetch/array
    ├── member.rs   — acesso a propriedades, estado mutex (convert_member_expr)
    ├── arrow.rs    — arrow functions → closures (convert_arrow_expr)
    ├── literal.rs  — literais, expressões de objeto/array/template
    └── misc.rs     — atribuições, atualizações, optional chaining
```

Todo código Rust é gerado utilizando macros `quote!` produzindo `proc_macro2::TokenStream` — nunca concatenação de strings.

## 📄 Licença

Licença MIT. Consulte [LICENSE](LICENSE) para detalhes.
