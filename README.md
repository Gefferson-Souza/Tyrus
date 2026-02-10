# Tyrus

> **Transpiler TypeScript → Rust**
> _Academic Project: High-Performance Source-to-Source Compiler_

![CI Status](https://github.com/gefferson-souza/Tyrus/actions/workflows/ci.yml/badge.svg)
![Rust Version](https://img.shields.io/badge/rust-1.75%2B-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

**Tyrus** (formerly _Oxidizer_) é uma ferramenta experimental de engenharia de software projetada para analisar código TypeScript moderno (incluindo padrões NestJS) e transpilar para código Rust idiomático, seguro e performático.

O objetivo não é suportar 100% da especificação TypeScript, mas sim definir um subconjunto **"Oxidizable Standard"** que permite escrever backends robustos em TS e compilá-los para um binário nativo.

## 🚀 Filosofia

1. **Safety First:** Se o código TS é inseguro (`any`, `eval`), o Tyrus rejeita a compilação.
2. **Idiomatic Output:** Não geramos "JavaScript em Rust". Geramos Rust real (`Result`, `Option`, `Structs`, `Tokio`).
3. **Opinionated:** Focamos em arquitetura backend moderna (Controller/Service/Repository).

## 📦 Funcionalidades Suportadas (The Oxidizable Standard)

### Estruturas de Dados

- [x] `interface` → `struct` (com `serde::Serialize/Deserialize`)
- [x] `class` DTOs → `struct`

### Tipagem

- [x] Primitivos: `string`, `number` (`f64`), `boolean`
- [x] Coleções: `Array<T>` → `Vec<T>`
- [x] Opcionais: `T | undefined` → `Option<T>`
- [x] Generics: `Box<T>` → `Box<T>`

### Lógica

- [x] `async/await` → `async fn` / `.await`
- [x] `if/else`
- [x] `while` loops
- [x] Operações matemáticas básicas
- [x] Métodos de Array: `map`, `filter`, `push`, `join`
- [x] Manipulação de String: `replace`, `split`, `trim`, `toUpperCase`

### Frameworks & I/O

- [x] **NestJS Controllers:** `@Get`, `@Post`, `@Body` → `Axum Handlers`
- [x] **HTTP Client:** `axios.get`, `fetch` → `reqwest`
- [x] **JSON:** `JSON.stringify/parse` → `serde_json`

## 🛠 Instalação

Pré-requisitos: Rust 1.75+ e Cargo.

```bash
# Clone o repositório
git clone https://github.com/gefferson-souza/Tyrus.git
cd Tyrus

# Compile o projeto
cargo build --release

# O binário estará em ./target/release/tyrus
```

## 📖 Uso

### Verificar compatibilidade (Check)

Analisa o projeto e aponta erros ou violações do padrão Oxidizable.

```bash
./target/release/tyrus check ./path/to/project/index.ts
```

### Compilar (Build)

Gera o código Rust na pasta `tyrus_output`.

```bash
./target/release/tyrus build ./path/to/project/index.ts
```

Ao final, você terá um novo projeto Rust completo. Basta entrar na pasta e rodar `cargo run`.

## 🤝 Contribuição

Este é um projeto acadêmico e open-source. Contribuições são bem-vindas, desde que sigam o `CODE_OF_CONDUCT.md` e as diretrizes em `CONTRIBUTING.md`.

## 📄 Licença

MIT License - Veja [LICENSE](LICENSE) para detalhes.
