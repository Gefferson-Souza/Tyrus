//! Driver Node × hospedado × promovido: roda cada sonda dos dois lados e emite o
//! veredito fechado em `equivalence.json` (D21). Só o oráculo decide vereditos.
//!
//! Contrato do alvo: recebe o caminho da sonda `.ts` como último argumento e herda
//! `NODE_OPTIONS=--require <shim>`. Um hospedeiro que ignora o shim fica visível em
//! `shim_canary`; o formato final (arquivo por sonda do `tyrus pack`) é decisão do dono.
pub mod classify;
pub mod corpus;
pub mod decide;
pub mod outcome;
pub mod profile;
pub mod record;
pub mod run;
pub mod runner;
