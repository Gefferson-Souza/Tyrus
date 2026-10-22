'use strict';
const util = require('util');
const { unit, int, pick, safe } = require('./prng.js');
const emit = require('./emit.js');
const LOCALES = ['en-US', 'pt-BR', 'de-DE', 'ja-JP', 'ar-EG', 'hi-IN', 'tr-TR'];
const STRINGS = ["it's", 'say "hi"', 'a`b\'c"', 'tab\there', 'nl\nx', 'é', '日本', '😀', '\ud83d', '\u0000', '', 'x'.repeat(90)];
const randomValue = (depth) => {
  const k = int(depth > 2 ? 6 : 10);
  if (k === 0) return pick(STRINGS);
  if (k === 1) return (unit() - 0.5) * 10 ** (int(30) - 10);
  if (k === 2) return pick([true, false, null, undefined, -0, NaN, 10n, 2n ** 70n]);
  if (k === 3) return Symbol(pick(['a', 'b c', '']));
  if (k === 4) return new Date(Math.floor(unit() * 4e12));
  if (k === 5) return pick([/a/g, /b\/c/iu, () => 1, function named() {}, class K {}]);
  if (k === 6 || k === 7) return Array.from({ length: int(12) }, () => randomValue(depth + 1));
  if (k === 8) return Object.fromEntries(Array.from({ length: int(6) }, (_, i) => [pick(['k' + i, 'a-b', 'x y', '3', 'ok_']), randomValue(depth + 1)]));
  return new Map(Array.from({ length: int(4) }, (_, i) => [i, randomValue(depth + 1)]));
};
const out = {};
const make = (name, n, gen) => { out[name] = Array.from({ length: n }, () => safe(gen)); };
make('inspectRandom', 1500, () => util.inspect(randomValue(0), { breakLength: pick([40, 80, 120]), compact: pick([3, 1, true, false]), depth: int(4) }));
make('inspectNumbers', 1500, () => util.inspect([(unit() - 0.5) * 10 ** (int(40) - 15), Math.round(unit() * 1e6) / 100, -(2 ** int(60))]));
make('inspectArrays', 800, () => util.inspect(Array.from({ length: 1 + int(40) }, () => pick([int(1000), 'ab', 'abcdefgh', 1.5, int(10) ** 6]))));
make('bufferEncodings', 2000, () => {
  const bytes = Buffer.from(Array.from({ length: int(24) }, () => (unit() < 0.4 ? 0x80 + int(128) : int(256))));
  const enc = pick(['hex', 'base64', 'base64url', 'latin1', 'ascii', 'utf8', 'utf16le', 'binary']);
  return enc + ':' + JSON.stringify(bytes.toString(enc)) + ':' + Buffer.from(bytes.toString(enc), enc).toString('hex');
});
make('bufferFromString', 1500, () => {
  const s = Array.from({ length: int(10) }, () => String.fromCharCode(unit() < 0.2 ? 0xd800 + int(0x800) : unit() < 0.3 ? 0x80 + int(0x700) : 0x20 + int(0x5f))).join('');
  return [Buffer.byteLength(s), Buffer.from(s).toString('hex'), Buffer.from(s, 'utf16le').toString('hex'), Buffer.from(s, 'latin1').toString('hex')].join('|');
});
make('bufferNumbers', 1500, () => {
  const b = Buffer.alloc(16);
  b.writeDoubleLE((unit() - 0.5) * 1e10, 0); b.writeFloatBE(unit() * 1e5, 8); b.writeInt32LE(int(2 ** 32) - 2 ** 31, 12);
  return [b.toString('hex'), b.readDoubleLE(0), b.readFloatBE(8), b.readBigUInt64LE(8), b.readIntBE(1, 5), b.indexOf(0xff), b.includes(Buffer.from([0, 0]))].join('|');
});
make('bufferErrors', 40, () => pick([() => Buffer.alloc(-1), () => Buffer.from({}), () => Buffer.alloc(4).readUInt32LE(2), () => Buffer.alloc(4).writeUInt8(300, 0), () => new TextDecoder('utf-8', { fatal: true }).decode(Buffer.from([0xff])), () => atob('***'), () => Buffer.from('abc', 'nope'), () => Buffer.concat([1]), () => new TextDecoder('nope')])());
make('textCodec', 1500, () => { const b = Uint8Array.from({ length: int(12) }, () => (unit() < 0.4 ? 0x80 + int(128) : int(256))); return [new TextDecoder().decode(b), new TextDecoder('utf-16le').decode(b), new TextDecoder('latin1').decode(b), btoa(String.fromCharCode(...b))].map((s) => JSON.stringify(s)).join('|'); });
make('toLocaleString', 2000, () => {
  const k = int(6); const loc = pick(LOCALES);
  if (k === 0) return ((unit() - 0.5) * 10 ** int(12)).toLocaleString(loc);
  if (k === 1) return (BigInt(int(2 ** 30)) * 1000003n).toLocaleString(loc);
  if (k === 2) return new Date(Math.floor(unit() * 4e12)).toLocaleString(loc, { timeZone: pick(['UTC', 'America/Sao_Paulo', 'Asia/Kolkata']) });
  if (k === 3) return new Date(Math.floor(unit() * 4e12)).toLocaleDateString(loc, { timeZone: 'UTC', month: 'long', weekday: 'short' });
  if (k === 4) return [1234.5, new Date(0), null, 'x', 2n ** 40n].toLocaleString(loc);
  return pick(['i', 'I', 'ß', 'ǆ', 'ŉ', 'İ']).toLocaleUpperCase(loc.slice(0, 2)) + pick(['I', 'İ', 'Σ']).toLocaleLowerCase(loc.slice(0, 2));
});
out.timersApi = [
  safe(() => { const t = setTimeout(() => {}, 1); const r = [t.constructor.name, t.hasRef(), t.unref() === t, t.hasRef(), t.ref() === t, typeof t.refresh, typeof t[Symbol.toPrimitive](), t.close() === t]; clearTimeout(t); return r.join(','); }),
  safe(() => { const i = setImmediate(() => {}); const r = [i.constructor.name, i.hasRef(), i.unref() === i]; clearImmediate(i); return r.join(','); }),
  safe(() => { const t = setInterval(() => {}, 1); const r = [t.constructor.name, typeof t.refresh]; clearInterval(t); return r.join(','); }),
  safe(() => setTimeout('not a function', 1)),
  safe(() => setImmediate(42)),
  safe(() => require('timers/promises').setTimeout.length + ',' + require('timers/promises').setImmediate.length),
  safe(() => require('timers').promises === require('timers/promises')),
  safe(() => util.promisify(setTimeout) === require('timers/promises').setTimeout),
  safe(() => util.inspect(setTimeout).slice(0, 80)),
];
emit(out);
