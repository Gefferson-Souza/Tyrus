interface Cfg { a: number; b: number; }
const base: Cfg = { a: 1, b: 1 };
const c: Cfg = { b: 3, ...base };
console.log(c.b);
