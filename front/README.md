# front: ST0, emit com o TypeScript do próprio projeto

Sidecar de build (Node, só em tempo de build; produção roda só o binário). Resolve o `typescript` **do projeto analisado**, lê o tsconfig pela Compiler API, emite arquivo a arquivo com as opções do projeto (só `outDir` muda) e escreve `facts.json`.

```sh
node front/src/cli.ts --project <raiz do projeto> --out <diretório> [--tsconfig tsconfig.json]
```

Saída em `<out>/emit/` (JS emitido) e `<out>/facts.json` (validado por `schemas/facts.schema.json`: versão do TypeScript, hash do tsconfig, opções efetivas de emit, inventário de fontes e saídas com sha256; AST e símbolos são do M2).

| Saída | Significado |
|---|---|
| 0 | emitido, sem erro de diagnóstico |
| 1 | emitido, mas o TypeScript do projeto reportou erros (mesma política do `tsc`: o emit acontece) |
| 2 | recusado: `TYRUS-E7001` (projeto sem `typescript` local), tsconfig ilegível ou uso incorreto |

Sem dependências npm: o Node de referência (`oracle/oracle.lock`) executa os `.ts` removendo tipos, então só sintaxe apagável (sem `enum`, `namespace`, parâmetros com propriedade) e imports com extensão `.ts`.

## Testes

```sh
node --test front/test/        # ou: ./scripts/gates.sh front
```

A matriz de versões está em `test/ts-matrix.json` (5.1.6, 5.9.3, 6.0.3). Cada versão e a réplica (`oracle/apps/replica`) são instaladas uma vez em `TYRUS_TS_MATRIX_DIR` (padrão `~/.local/share/tyrus-ts-matrix`, nunca `/tmp`) e o emit do sidecar é comparado byte a byte com `tsc -p` da mesma versão. O `outDir` do controle precisa ter a mesma profundidade do sidecar, porque os source maps embutem o caminho relativo até as fontes.
