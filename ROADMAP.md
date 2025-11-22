# 🗺️ Oxidizer Roadmap

## 🏁 Milestone 0: The Foundation (Current)
- [x] Project Scaffolding (Cargo Workspace)
- [x] CLI Basic Structure (`clap`)
- [x] **Tracer Bullet:** Connect CLI to Parser (Read a file and print "Success")
- [ ] CI/CD Pipeline Setup (GitHub Actions)

## � Milestone 1: The Analyzer (Complete)
- [x] **Analyzer Core:** Implement `ox_analyzer` crate
- [x] **Linter Rules:**
    - [x] Block `any`
    - [x] Block `eval`
    - [x] Block `var`
- [x] **Refactoring:** Compliance with Guidelines (Newtype, Tracing, Tests)
- [x] Implement basic Lints:
    - [x] Ban `any` type
    - [x] Ban `eval`
    - [x] Check for `var` usage
- [ ] Error Reporting with `miette` (Visual spans)

## 🏁 Milestone 2: The Type Transpiler (Complete)
- [x] Convert TS `interface` -> Rust `struct`
- [x] Map primitive types (`string` -> `String`, `number` -> `f64`)
- [x] Auto-derive `Serialize, Deserialize` (Serde)
- [x] Output `.rs` files using `quote!`

## 🏁 Milestone 3: Logic & Functions (Complete)
- [x] Convert simple `fn` declarations
- [x] Basic control flow (`if`, `return`)
- [x] Binary expressions (Math)

## 🚀 Milestone 4: Advanced Features (The "Viral" Stuff)
- [ ] Async/Await support (`tokio`)
- [ ] Class to Struct conversion (Inheritance simulation)
- [ ] HTTP Client mapping (`axios` -> `reqwest`)