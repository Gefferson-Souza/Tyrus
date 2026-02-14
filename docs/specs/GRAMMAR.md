# Tyrus Grammar Specification (Oxidizable Subset)

This document formally defines the subset of TypeScript supported by Tyrus using EBNF notation.

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

## 3. Expressions

```ebnf
Expression ::=
    | BinaryExpr
    | UnaryExpr
    | CallExpr
    | MemberExpr
    | Literal

UnaryExpr ::= ('!' | '-' | '+') Expression
```

## 4. Types

```ebnf
Type ::=
    | 'string'
    | 'number'
    | 'boolean'
    | Identifier  (* User defined structs *)
    | Type '[]'   (* Array *)
    | 'Promise' '<' Type '>'
```

## 5. Control Flow (Supported)

```ebnf
WhileStmt ::= 'while' '(' Expression ')' Block
IfStmt ::= 'if' '(' Expression ')' Block ('else' Block)?
```

## 6. Unsafe (Forbidden)

The following constructs are explicitly rejected by the `tyrus_analyzer`:

- `any` type usage.
- `eval()` calls.
- `var` declarations (use `let`/`const`).
