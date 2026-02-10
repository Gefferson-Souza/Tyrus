# 3. Mapeamento de Generics (TypeScript -> Rust)

Data: 2026-02-10
Status: Aceito

## Contexto

TypeScript possui um sistema de tipos estrutural com generics muito flexíveis (`any`, restrições parciais). Rust possui um sistema nominal e monomorfização.
Precisamos permitir que usuários definam classes e funções genéricas em TS que compilem em Rust.

## Decisão

Mapearemos Generics do TS para Generics do Rust com Restrições de Trait (Trait Bounds) padrão.

### Regras de Mapeamento:

1.  **Declaração:**
    - TS: `class Box<T> { ... }`
    - Rust: `struct Box<T> { ... }`

2.  **Trait Bounds Automáticos:**
    - Todo parâmetro genérico `T` em Rust receberá automaticamente:
      `T: serde::Serialize + serde::Deserialize + Clone + Debug + Default`
    - _Justificativa:_ Backends precisam serializar dados (JSON), clonar estados e debugar. Sem esses traits, o uso de `T` seria muito restrito.

3.  **PhantomData:**
    - Se um parâmetro `T` é declarado mas não usado nos campos da struct:
    - Rust: Adicionar campo `_phantom: std::marker::PhantomData<T>`.
    - _Justificativa:_ O compilador Rust rejeita parâmetros genéricos não utilizados.

4.  **Herança de Generics:**
    - Não suportaremos restrições complexas do TS (`T extends keyof U`) na v1. Elas serão tratadas como `T` simples.

## Consequências

### Positivas

- Permite criar DTOs reutilizáveis (`ApiResponse<T>`).
- Garante que os tipos genéricos sejam úteis (serializáveis).

### Negativas

- Restrição excessiva: Nem todo `T` precisa ser `Default`, mas estamos forçando. Isso pode impedir o uso de tipos que não implementam `Default`.
- _Mitigação:_ No futuro, podemos analisar o uso do tipo para relaxar os bounds.
