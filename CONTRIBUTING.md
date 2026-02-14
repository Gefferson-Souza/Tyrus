# Contribuindo para o Tyrus

Obrigado por seu interesse em contribuir! Este é um projeto acadêmico e open-source.

## 🛠 Setup do Ambiente

1. **Rust:** Instale via [rustup.rs](https://rustup.rs). Versão mínima 1.75.
2. **Dependências:** O projeto usa `cargo`.
3. **Editor:** Recomendamos VS Code com a extensão `rust-analyzer`.

## 🧪 Rodando Testes

O projeto utiliza um harness personalizado (`tyrus_test_utils`) que garante que todo código gerado seja compilável.

```bash
# Rodar a suite completa (Unitários + Integração + Snapshots)
cargo test --workspace

# Se houver snapshots novos (e corretos), atualize-os:
cargo insta review
# Ou aceite automaticamente se tiver certeza:
cargo insta test --accept
```

## 🧹 Linting e Formatação

O CI irá falhar se o código não estiver formatado ou tiver warnings.

```bash
cargo fmt
cargo clippy --workspace -- -D warnings
```

## 📝 Processo de Pull Request

1. Fork o projeto.
2. Crie uma branch (`git checkout -b feature/minha-feature`).
3. Comite suas mudanças seguindo [Conventional Commits](https://www.conventionalcommits.org/) (ex: `feat: implement while loops`).
4. Abra um PR para a branch `main`.
5. Aguarde a revisão.

## ⚖️ Padrões de Código

Consulte `Guidelines.md` para entender as regras de engenharia (Newtypes, Visitor Pattern, Error Handling).
