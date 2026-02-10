# TypeRust Grammar Specification (Oxidizable subset)

This document formally defines the subset of TypeScript supported by TypeRust using EBNF notation.

## 1. Fundamentals

```ebnf
Program ::= Statement*

Statement ::=
    | InterfaceDecl
    | ClassDecl
    | FunctionDecl
    | VariableDecl
    | ExpressionStmt
    | ReturnStmt
    | IfStmt
    | WhileStmt
```

## 2. Declarations

```ebnf
InterfaceDecl ::= 'interface' Identifier '{' InterfaceMember* '}'
InterfaceMember ::= Identifier '?'? ':' Type ';'

ClassDecl ::= 'class' Identifier '{' ClassMember* '}'
ClassMember ::= PropertyDecl | MethodDecl

FunctionDecl ::= 'async'? 'function' Identifier '(' ParamList ')' ':' Type '{' Block '}'
```

## 3. Types

```ebnf
Type ::=
    | 'string'
    | 'number'
    | 'boolean'
    | Identifier  (* User defined structs *)
    | Type '[]'   (* Array *)
    | 'Promise' '<' Type '>'
```

## 4. Control Flow (Supported)

```ebnf
WhileStmt ::= 'while' '(' Expression ')' Block
IfStmt ::= 'if' '(' Expression ')' Block ('else' Block)?
```

## 5. Unsafe (Forbidden)

The following constructs are explicitly rejected by the `typerust_analyzer`:

- `any` type usage.
- `eval()` calls.
- `var` declarations (use `let`/`const`).
