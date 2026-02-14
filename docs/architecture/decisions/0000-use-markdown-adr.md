# 0. Uso de Architecture Decision Records (ADR)

Data: 2025-11-22
Status: Aceito

## Contexto
Precisamos registrar decisões arquiteturais significativas para o projeto Tyrus.
Como este projeto visa ser uma ferramenta de engenharia complexa (compilador) e um artefato acadêmico, é crucial manter um histórico do "porquê" e "como" as decisões foram tomadas.
A falta de documentação sobre decisões de design pode levar a retrabalho, perda de contexto e dificuldade na defesa da tese de mestrado.

## Decisão
Adotaremos o formato **Architecture Decision Records (ADR)**.
Usaremos arquivos Markdown numerados sequencialmente na pasta `docs/architecture/decisions`.

Cada ADR deve seguir esta estrutura:
1. **Título:** Curto e descritivo.
2. **Status:** Proposto, Aceito, Depreciado ou Rejeitado.
3. **Contexto:** Qual é o problema que estamos tentando resolver? Quais são as restrições?
4. **Decisão:** O que vamos fazer? (Tecnologia, Padrão, Algoritmo).
5. **Consequências:** O que ganhamos (pros) e o que perdemos/pagamos (contras) com essa decisão.

## Consequências
### Positivas
- Histórico claro da evolução do projeto.
- Facilita o onboarding de novos contribuidores (Open Source).
- Material pronto para a escrita da dissertação de mestrado.

### Negativas
- Requer disciplina para escrever o documento antes ou durante a implementação.