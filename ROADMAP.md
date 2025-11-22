# 🗺️ Oxidizer Roadmap

## 🏁 Milestone 0: The Foundation (Current)
- [x] Project Scaffolding (Cargo Workspace)
- [x] CLI Basic Structure (`clap`)
- [x] **Tracer Bullet:** Connect CLI to Parser (Read a file and print "Success")
- [ ] CI/CD Pipeline Setup (GitHub Actions)

## 🏗️ Milestone 1: The Analyzer (Focus: "oxidizer check")
- [ ] Implement `swc` integration to parse TypeScript to AST
- [ ] Create Dependency Graph (Resolve `import` statements)
- [ ] Implement basic Lints:
    - [ ] Ban `any` type
    - [ ] Ban `eval`
    - [ ] Check for `var` usage
- [ ] Error Reporting with `miette` (Visual spans)

## ⚙️ Milestone 2: The Type Transpiler (Focus: DTOs)
- [ ] Convert TS `interface` -> Rust `struct`
- [ ] Map primitive types (`string` -> `String`, `number` -> `f64`)
- [ ] Auto-derive `Serialize, Deserialize` (Serde)
- [ ] Output `.rs` files using `quote!`

## 🧠 Milestone 3: Logic & Functions
- [ ] Convert simple `fn` declarations
- [ ] Basic control flow (`if`, `return`)
- [ ] Binary expressions (Math)

## 🚀 Milestone 4: Advanced Features (The "Viral" Stuff)
- [ ] Async/Await support (`tokio`)
- [ ] Class to Struct conversion (Inheritance simulation)
- [ ] HTTP Client mapping (`axios` -> `reqwest`)