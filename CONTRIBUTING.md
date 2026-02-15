# Contributing to Tyrus

## 🌳 Git Workflow: The "Tyrus Pattern"

We follow a strict **Feature Branch Workflow** combined with **Conventional Commits**.

### Branching Strategy

- **`main`**: Protected. Production-ready code only. No direct commits.
- **`feat/`**: New features (e.g., `feat/async-await`, `feat/new-parser`).
- **`fix/`**: Bug fixes (e.g., `fix/memory-leak`, `fix/cli-panic`).
- **`chore/`**: Maintenance, config, docs (e.g., `chore/optimize-workflow`, `docs/update-readme`).
- **`refactor/`**: Code restructuring without behavior change.

### 📝 Commit Convention

We use [Conventional Commits](https://www.conventionalcommits.org/).

**Format:** `<type>(<scope>): <subject>`

**Types:**

- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation only changes
- `style`: Changes that do not affect the meaning of the code (white-space, formatting, etc)
- `refactor`: A code change that neither fixes a bug nor adds a feature
- `perf`: A code change that improves performance
- `test`: Adding missing tests or correcting existing tests
- `chore`: Changes to the build process or auxiliary tools and libraries such as documentation generation

**Examples:**

- `feat(codegen): implement structural typing for interfaces`
- `fix(cli): resolve panic when input file is missing`
- `chore(deps): upgrade axum to v0.7`

## 🚀 Pull Request Process

1.  Create a branch complying with the strategy above.
2.  Ensure `cargo test --workspace` passes locally.
3.  Run `cargo clippy` and fix warnings.
4.  Open a PR to `main`.
5.  Fill out the **PR Template** completely.
6.  Wait for CI checks to pass and request review.
