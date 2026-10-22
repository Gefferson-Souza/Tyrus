#!/bin/sh
# Verifica o ADR 0019 (ferramental do oráculo): seções, linhas da tabela com versão, opções A-D, índice e links.
set -eu
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ADR="${ADR_0019_PATH:-$ROOT/docs/architecture/decisions/0019-oracle-and-harness-tooling.md}"
INDEX="$ROOT/docs/ARCHITECTURE.md"
fail=0
err() { echo "check_adr_0019: $*" >&2; fail=1; }

[ -f "$ADR" ] || { echo "check_adr_0019: ADR ausente: $ADR" >&2; exit 1; }

grep -q '^Status: Proposto' "$ADR" || err "status deve ser Proposto"
for section in '## Alternativas rejeitadas' '## Gatilhos de revisão' '## Divergências do RFC' '### 5. Supply-chain coverage'; do
    grep -q "^$section\$" "$ADR" || err "seção ausente: $section"
done

# Cada linha da tabela da Decisão 1 (a..i) precisa carregar a ferramenta e a versão exata na MESMA linha da tabela.
check_row() {
    row="$1"; pattern="$2"; label="$3"
    grep -E "^\| $row \|" "$ADR" | grep -Eq "$pattern" || err "linha $row ($label) sem o padrão: $pattern"
}
check_row a 'python3 -I.*tested with \*\*3\.12\.3\*\*' 'Python'
check_row b '26\.11\.1.*22\.21\.1' 'Node'
check_row c 'cargo-mutants.*27\.1\.0' 'cargo-mutants'
check_row d 'jsonschema.*\*\*0\.59\.0\*\*' 'jsonschema'
check_row e 'npm.*11\.20\.0' 'npm'
check_row f 'Docker.*29\.9\.1.*17\.11.*sha256:b0f9560a2de083e2cc7382e75f808c7381a32852a7ec49117deedb300e552b24' 'Docker/Postgres'
check_row g 'cargo-nextest.*0\.9\.143.*CI-floating' 'nextest'
check_row h 'shellcheck.*CI-floating' 'shellcheck'
check_row i 'Bun.*1\.2\.22' 'Bun'
grep -q '^| # | Tool | Version | Pin |' "$ADR" || err "coluna Pin ausente na tabela da Decisão 1"
grep -q 'setup-node@v7' "$ADR" || err "setup-node@v7 ausente"

# Cada bump de Cargo.lock precisa de linha na tabela da Decisão 4 com versão antes e depois.
check_bump() {
    grep -Eq "^\| \`$1\` \| $2 \| $3 \|" "$ADR" || err "bump $1 sem $2 -> $3 na tabela"
}
check_bump zmij '1\.0\.20' '1\.0\.23'
check_bump regex-automata '0\.4\.14' '0\.4\.18'
check_bump num-bigint '0\.4\.6' '0\.4\.8'
for opt in 'A' 'B' 'C' 'D'; do
    grep -Eq "^\| (\*\*)?$opt[ :(]" "$ADR" || err "opção $opt ausente na tabela de opções da Decisão 4"
done

grep -q '0019-oracle-and-harness-tooling.md' "$INDEX" || err "docs/ARCHITECTURE.md não indexa o 0019"

dir="$(dirname "$ADR")"
for link in $(grep -oE '\]\(([^)#]+)(#[^)]*)?\)' "$ADR" | sed -E 's/^\]\(//;s/\)$//;s/#.*//' | grep -vE '^(https?:|$)'); do
    [ -e "$dir/$link" ] || err "link quebrado: $link"
done

[ "$fail" -eq 0 ] && echo "check_adr_0019: ok"
exit "$fail"
