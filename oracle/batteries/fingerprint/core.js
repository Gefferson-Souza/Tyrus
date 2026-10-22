const MASK = (1n << 64n) - 1n;
let state = 0x243F6A8885A308D3n;
function next() {
  state = (state + 0x9E3779B97F4A7C15n) & MASK;
  let z = state;
  z = ((z ^ (z >> 30n)) * 0xBF58476D1CE4E5B9n) & MASK;
  z = ((z ^ (z >> 27n)) * 0x94D049BB133111EBn) & MASK;
  return z ^ (z >> 31n);
}
const unit = () => Number(next() >> 11n) / 2 ** 53;
const f64 = new Float64Array(1);
const u64 = new BigUint64Array(f64.buffer);
const bits = (x) => { f64[0] = x; return u64[0].toString(16); };
const tenPow = (k) => Number('1e' + k);
const N = 20000;
const numericCases = {
  pow: () => Math.pow(unit() * 100, unit() * 40 - 20),
  powOperator: () => { const b = unit() * 100; const e = unit() * 40 - 20; return b ** e; },
  powIntExp: () => Math.pow(unit() * 10, Math.floor(unit() * 30) - 15),
  sin: () => Math.sin(unit() * 2000 - 1000),
  cos: () => Math.cos(unit() * 2000 - 1000),
  tan: () => Math.tan(unit() * 2000 - 1000),
  exp: () => Math.exp(unit() * 200 - 100),
  log: () => Math.log(unit() * 1e6),
  cbrt: () => Math.cbrt(unit() * 2e6 - 1e6),
  atan2: () => Math.atan2(unit() * 200 - 100, unit() * 200 - 100),
  sinh: () => Math.sinh(unit() * 40 - 20),
  expm1: () => Math.expm1(unit() * 40 - 20),
  log1p: () => Math.log1p(unit() * 1e3),
  hypot: () => Math.hypot(unit() * 1e3, unit() * 1e3, unit() * 1e3),
};
const textCases = {
  numberToString: () => String(unit() * tenPow(Math.floor(unit() * 40) - 20)),
  toFixed: () => (unit() * 1e6 - 5e5).toFixed(Math.floor(unit() * 20)),
  toPrecision: () => (unit() * 1e6).toPrecision(1 + Math.floor(unit() * 21)),
  toExponential: () => (unit() * 1e9).toExponential(Math.floor(unit() * 20)),
  jsonFloat: () => JSON.stringify([unit() * 1e15, unit() / 1e10]),
  isoDate: () => new Date(Math.floor(unit() * 4e12) - 1e12).toISOString(),
  parseFloat: () => String(parseFloat((unit() * 1e8).toString(36))),
  sortCalls: () => {
    const size = 2 + Math.floor(unit() * 60);
    const values = Array.from({ length: size }, () => Math.floor(unit() * 20));
    const calls = [];
    values.sort((a, b) => { calls.push(a + ':' + b); return a - b; });
    return calls.join(',');
  },
};
const out = {};
for (const [name, gen] of Object.entries(numericCases)) {
  const values = [];
  for (let i = 0; i < N; i++) values.push(bits(gen()));
  out[name] = values;
}
for (const [name, gen] of Object.entries(textCases)) {
  const values = [];
  for (let i = 0; i < 2000; i++) values.push(gen());
  out[name] = values;
}
const thrown = (fn) => { try { fn(); return 'no throw'; } catch (e) { return e.constructor.name + ': ' + e.message; } };
out.errorMessages = [
  thrown(() => null.x),
  thrown(() => undefined.foo.bar),
  thrown(() => { const o = {}; o.method(); }),
  thrown(() => { const n = 1; n(); }),
  thrown(() => (1).toFixed(200)),
  thrown(() => new Array(-1)),
  thrown(() => JSON.parse('{')),
  thrown(() => JSON.parse('{"a":1,}')),
  thrown(() => BigInt(1.5)),
  thrown(() => Symbol() + ''),
  thrown(() => { const c = 1; eval('c = 2'); }),
  thrown(() => Object.defineProperty(Object.freeze({}), 'x', { value: 1 })),
  thrown(() => { 'use strict'; Object.freeze([1]).push(2); }),
  thrown(() => decodeURIComponent('%')),
  thrown(() => new (class A { #x = 1; static read(o) { return o.#x; } }).constructor.read({})),
  thrown(() => [].reduce((a, b) => a + b)),
  thrown(() => 'abc'.repeat(-1)),
  thrown(() => structuredCloneMissing()),
];
out.__engine = typeof process !== 'undefined' && process.versions ? 'node ' + process.version + ' v8=' + process.versions.v8 : 'embedded';
const serialized = JSON.stringify(out);
if (typeof process !== 'undefined' && process.stdout) process.stdout.write(serialized);
serialized;
