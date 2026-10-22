# Baterias de conformidade do motor

Régua que diz **em quais classes um motor JavaScript difere do Node de referência**. Ela mede o motor; não mede o compilador, não executa o Tyrus e não alega equivalência de programa nenhum. Quem a usa: o SP1 (escolha da plataforma do piso), o gate de fingerprint e o critério E4 do M1.

## Conteúdo

| Pasta | O que é |
|---|---|
| `fingerprint/` | sondas que emitem JSON `{classe: [valores]}`: `core.js` (sonda semente, intocada), `intl.js`, `unicode.js`, `tz.js`, `tzlocal.js` (uma execução por fuso de `index.json`), `runtime.js` (`util.inspect`, `Buffer`, `TextDecoder`, `toLocaleString`, timers) e `tuple.js` (identifica o motor) |
| `litmus/` | litmus determinísticos; `L01` a `L30` de ordem (CJS, `.js`) e `L31` a `L36` de ESM (`.mjs`, mais dois `require(esm)` em `.js`), com módulos auxiliares em `fixtures/`; as 11 seções estão em `index.json` |
| `inspect/` | litmus de `util.inspect` e `console` (`I01` a `I16`) |
| `perturbation/` | scripts que flutuam de propósito; **não** são gate, só provam que o detector de instabilidade funciona |
| `mutants/` | motores defeituosos conhecidos (pré-carregados com `--require`); cada um declara em `mutants.json` as classes que têm de reprovar |
| `goldens/node-26.11.1/` | `meta.json` (tupla), `golden.json` (`n` e `sha256` por classe) e `outputs/` (texto legível dos litmus e do `inspect`) |
| `tests/` | `python3 -I -m unittest discover -s oracle/batteries/tests` |

## Como usar

```sh
export TYRUS_NODE_REFERENCE=<caminho do node v26.11.1>
python3 -I oracle/batteries/run.py --engine-cmd "$TYRUS_NODE_REFERENCE" --compare oracle/batteries/goldens/node-26.11.1
python3 -I oracle/batteries/run.py --engine-cmd "<motor> [flags]" --compare oracle/batteries/goldens/node-26.11.1 --class 'fingerprint.sin,litmus.L0*'
```

O script é anexado como último argumento do `--engine-cmd`. Códigos de saída: `0` tudo igual; `1` alguma classe diferente, com erro ou instável (lista em `diff_classes`); `2` erro do harness (motor inexistente, golden ausente ou desatualizado). `tuple_matches` é informativo: o critério é diff zero por classe, porque um motor embutido legitimamente se identifica de outro jeito.

`--stability N` repete cada litmus N vezes (padrão 1 no `--compare`, 5 no `--record`). `--record DIR` só grava se o motor tiver a tupla de `oracle/oracle.lock` e nenhum script flutuar.

## Como acrescentar uma classe

1. Fingerprint: gere valores com `prng.js` (nunca `Math.random`, relógio ou pid) e devolva-os em `emit(...)`. Litmus e `inspect`: arquivo novo; nada de espera de 10 ms ou mais para ordenar eventos; nada de ordem entre fontes de E/S independentes.
2. Litmus: inclua o nome em exatamente uma seção de `index.json`.
3. `python3 -I oracle/batteries/run.py --engine-cmd "$TYRUS_NODE_REFERENCE" --record oracle/batteries/goldens/node-26.11.1` (falha se algo flutuar; mova o script para `perturbation/`).
4. Revise o diff de `goldens/`: texto novo em `outputs/` é a regra, texto alterado exige explicação.
5. Se a classe pegar um defeito que nenhum mutante pega, acrescente o mutante.

## O que esta régua não prova

- Diff zero nas classes listadas não é equivalência de motor, só ausência de diferença nessas classes. O que não está na lista não foi medido.
- Os goldens provam que o Node de referência se reproduz nesta máquina; não provam nada sobre outro SO ou CPU até o CI rodar o mesmo comando.
- Trinta litmus são uma amostra de ordem, não uma prova de ordem.
- Nenhum número daqui entra em README, release ou PR sem passar pelo ledger.

## Proveniência

`fingerprint/core.js` é cópia byte a byte de `engine_probe.js` da sonda semente de 2026-10-13 (teste `test_seed_probe_is_preserved_verbatim` fixa o sha256). Os goldens foram gravados no Node de referência de `oracle/oracle.lock`. A seção S11 e o agrupamento em 11 seções são definições deste repositório (o `lit.cjs` original do red-team não foi recuperado).

## Ambiente de execução

- `http` e `net` (`L22`, `L23`) precisam de loopback ativo. Num namespace de rede novo (`unshare -n`) o loopback nasce desligado e seria preciso `ip link set lo up`; isso é comportamento geral do Linux e **não foi verificado** aqui (`unshare -rn` é negado nesta máquina).
- Os litmus usam porta efêmera (`listen(0, '127.0.0.1')`); as portas 3000 a 3002 não são tocadas.
