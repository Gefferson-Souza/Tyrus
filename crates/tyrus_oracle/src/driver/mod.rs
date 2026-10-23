//! Driver Node × hospedado × promovido: roda cada sonda dos dois lados e emite o
//! veredito fechado em `equivalence.json` (D21). Só o oráculo decide vereditos.
pub mod classify;
pub mod corpus;
pub mod decide;
pub mod outcome;
pub mod profile;
pub mod record;
pub mod run;
pub mod runner;
