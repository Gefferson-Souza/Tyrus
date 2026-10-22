'use strict';
const { unit, int, safe, fnv } = require('./prng.js');
const emit = require('./emit.js');
const PROPS = ['L', 'Lu', 'Ll', 'Lt', 'Lm', 'Lo', 'M', 'N', 'Nd', 'P', 'S', 'Z', 'C', 'Cn', 'Emoji', 'Emoji_Presentation', 'Extended_Pictographic', 'Script=Latin', 'Script=Han', 'Script=Greek', 'Script=Cyrillic', 'Script=Arabic', 'Script_Extensions=Latin', 'ID_Start', 'ID_Continue', 'White_Space', 'Alphabetic', 'Lowercase', 'Uppercase', 'Cased', 'Case_Ignorable', 'Assigned', 'Regional_Indicator', 'East_Asian_Width=Wide', 'General_Category=Decimal_Number'];
const blocks = [];
for (let start = 0; start <= 0x10ffff; start += 0x10000) {
  let text = '';
  for (let cp = start; cp < Math.min(start + 0x10000, 0x110000); cp++) if (cp < 0xd800 || cp > 0xdfff) text += String.fromCodePoint(cp);
  blocks.push({ start, text });
}
const countProp = (prop) => {
  const re = new RegExp('\\p{' + prop + '}', 'gu');
  let count = 0; let sum = 0;
  for (const { text } of blocks) for (const m of text.matchAll(re)) { count++; sum += m[0].codePointAt(0) % 65521; }
  return count + ':' + sum;
};
const hashBlocks = (fn) => blocks.map(({ start, text }) => { let h = ''; for (const ch of text) h += fn(ch); return start.toString(16) + ':' + fnv(h); });
const out = {};
out.unicodeProps = PROPS.map((p) => p + '=' + safe(() => countProp(p)));
out.unicodeUpper = hashBlocks((c) => c.toUpperCase());
out.unicodeLower = hashBlocks((c) => c.toLowerCase());
out.unicodeLocaleUpper = ['tr', 'lt', 'az', 'el'].map((l) => l + ':' + fnv(blocks.map(({ text }) => text.toLocaleUpperCase(l)).join('')));
out.unicodeNormalize = ['NFC', 'NFD', 'NFKC', 'NFKD'].map((f) => f + ':' + fnv(blocks.map(({ text }) => text.normalize(f)).join('')));
out.unicodeRegexCase = hashBlocks((c) => (/[a-zß-ÿΑ-ωЀ-ӿ]/iu.test(c) ? '1' : '0'));
out.unicodeCompare = Array.from({ length: 2000 }, () => { const a = String.fromCodePoint(0x20 + int(0xd000)); const b = String.fromCodePoint(0x20 + int(0xd000)); return ['en', 'de', 'sv', 'ja', 'tr'].map((l) => a.localeCompare(b, l)).join(','); });
out.unicodeGraphemes = Array.from({ length: 500 }, () => { let s = ''; for (let i = 0; i < 6; i++) s += String.fromCodePoint(unit() < 0.5 ? 0x1f300 + int(0x300) : 0x300 + int(0x70)) + (unit() < 0.2 ? '‍' : ''); return [...new Intl.Segmenter('en', { granularity: 'grapheme' }).segment(s)].length + ':' + s.length; });
out.unicodeIdentifier = Array.from({ length: 1000 }, () => { const cp = int(0x30000); return (/^[\p{ID_Start}$_][\p{ID_Continue}$‌‍]*$/u.test(String.fromCodePoint(cp)) ? 1 : 0) + ':' + safe(() => String.fromCodePoint(cp).normalize('NFKC').length); });
emit(out);
