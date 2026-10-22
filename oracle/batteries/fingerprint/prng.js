'use strict';
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
const int = (n) => Math.floor(unit() * n);
const pick = (list) => list[int(list.length)];
const f64 = new Float64Array(1);
const u64 = new BigUint64Array(f64.buffer);
const bits = (x) => { f64[0] = x; return u64[0].toString(16); };
const safe = (fn) => {
  try { return String(fn()); } catch (e) { return 'THROW ' + e.constructor.name + ': ' + e.message; }
};
const fnv = (text) => {
  let h = 0x811c9dc5;
  for (let i = 0; i < text.length; i++) { h ^= text.charCodeAt(i); h = Math.imul(h, 0x01000193) >>> 0; }
  return h.toString(16);
};
module.exports = { unit, int, pick, bits, safe, fnv };
