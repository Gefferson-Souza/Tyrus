'use strict';
const { unit, int, pick, safe } = require('./prng.js');
const emit = require('./emit.js');
const LOCALES = ['en-US', 'pt-BR', 'de-DE', 'fr-FR', 'ja-JP', 'ar-EG', 'hi-IN', 'ru-RU', 'tr-TR', 'sv-SE', 'zh-CN', 'he-IL', 'th-TH', 'ko-KR'];
const ZONES = ['UTC', 'America/Sao_Paulo', 'Asia/Kolkata', 'Australia/Lord_Howe', 'America/St_Johns', 'Europe/Berlin', 'Asia/Tokyo', 'Pacific/Chatham', 'America/Argentina/Buenos_Aires', 'Africa/Cairo'];
const CURRENCIES = ['USD', 'BRL', 'EUR', 'JPY', 'INR', 'EGP', 'CHF'];
const UNITS = ['kilometer-per-hour', 'liter', 'byte', 'celsius', 'second', 'percent'];
const NUMBER_OPTIONS = [
  () => ({}),
  () => ({ style: 'currency', currency: pick(CURRENCIES) }),
  () => ({ style: 'percent', maximumFractionDigits: int(4) }),
  () => ({ notation: 'compact', compactDisplay: pick(['short', 'long']) }),
  () => ({ style: 'unit', unit: pick(UNITS), unitDisplay: pick(['short', 'long', 'narrow']) }),
  () => ({ notation: pick(['scientific', 'engineering']) }),
  () => ({ signDisplay: pick(['always', 'exceptZero', 'negative']), minimumIntegerDigits: 1 + int(4) }),
  () => ({ useGrouping: pick([true, false, 'min2', 'always']), minimumFractionDigits: int(3), maximumFractionDigits: 3 + int(3) }),
  () => ({ numberingSystem: pick(['latn', 'arab', 'deva', 'thai']) }),
];
const DATE_OPTIONS = [
  () => ({ dateStyle: pick(['full', 'long', 'medium', 'short']) }),
  () => ({ timeStyle: pick(['full', 'long', 'medium', 'short']) }),
  () => ({ dateStyle: 'full', timeStyle: 'long' }),
  () => ({ hour: 'numeric', minute: '2-digit', hour12: pick([true, false]) }),
  () => ({ weekday: 'long', era: 'short', year: 'numeric', month: 'long', day: 'numeric' }),
  () => ({ timeZoneName: pick(['short', 'long', 'shortOffset', 'longOffset', 'shortGeneric', 'longGeneric']), hour: 'numeric' }),
  () => ({ dayPeriod: 'long', hour: 'numeric' }),
  () => ({ fractionalSecondDigits: 3, second: 'numeric', minute: 'numeric' }),
  () => ({ calendar: pick(['gregory', 'buddhist', 'japanese', 'islamic-umalqura', 'persian']), dateStyle: 'long' }),
];
const WORDS = ['côté', 'cote', 'Côte', 'coté', 'z', 'a', 'Z', 'ä', 'ö', 'o', 'ñ', 'n', 'ß', 'ss', 'é', 'e', 'ç', 'c', 'Å', 'å', 'æ', 'İ', 'i', 'ı', 'I', '10', '9', '2', 'あ', 'ア', '漢', '日', 'Ж', 'а'];
const TEXTS = ['Olá, mundo! Tudo bem? Sim.', 'The quick brown fox. It jumped!', '日本語のテキストです。これは文です。', '👨‍👩‍👧‍👦 family 🇧🇷 flag é ZWJ', 'ไทยไม่มีช่องว่างระหว่างคำ', 'a-b c_d e.f 1,234.56'];
const T0 = Date.UTC(1950, 0, 1);
const T1 = Date.UTC(2045, 0, 1);
const ts = () => Math.floor(T0 + unit() * (T1 - T0));
const out = {};
const make = (name, n, gen) => { out[name] = Array.from({ length: n }, () => safe(gen)); };
make('intlNumber', 2000, () => new Intl.NumberFormat(pick(LOCALES), pick(NUMBER_OPTIONS)()).format((unit() - 0.5) * 10 ** int(12)));
make('intlNumberParts', 500, () => JSON.stringify(new Intl.NumberFormat(pick(LOCALES), { style: 'currency', currency: pick(CURRENCIES) }).formatToParts(unit() * 1e6)));
make('intlNumberRange', 300, () => new Intl.NumberFormat(pick(LOCALES), { maximumFractionDigits: 1 }).formatRange(int(100), 100 + int(1000)));
make('intlDate', 2000, () => new Intl.DateTimeFormat(pick(LOCALES), { timeZone: pick(ZONES), ...pick(DATE_OPTIONS)() }).format(ts()));
make('intlDateParts', 500, () => JSON.stringify(new Intl.DateTimeFormat(pick(LOCALES), { timeZone: pick(ZONES), dateStyle: 'medium', timeStyle: 'short' }).formatToParts(ts())));
make('intlDateRange', 300, () => { const a = ts(); return new Intl.DateTimeFormat(pick(LOCALES), { timeZone: 'UTC', dateStyle: 'medium' }).formatRange(a, a + int(40) * 86400000); });
make('intlPlural', 600, () => { const n = int(200) + (unit() < 0.3 ? 0.5 : 0); const loc = pick(LOCALES); return new Intl.PluralRules(loc, { type: pick(['cardinal', 'ordinal']) }).select(n) + '/' + new Intl.PluralRules(loc).resolvedOptions().pluralCategories.join(','); });
make('intlRelative', 600, () => new Intl.RelativeTimeFormat(pick(LOCALES), { numeric: pick(['auto', 'always']), style: pick(['long', 'short', 'narrow']) }).format(int(200) - 100, pick(['second', 'minute', 'hour', 'day', 'week', 'month', 'quarter', 'year'])));
make('intlList', 300, () => new Intl.ListFormat(pick(LOCALES), { type: pick(['conjunction', 'disjunction', 'unit']), style: pick(['long', 'short', 'narrow']) }).format(['um', 'dois', 'três', 'quatro'].slice(0, 1 + int(4))));
make('intlDisplayNames', 400, () => { const type = pick(['region', 'language', 'currency', 'script']); const code = { region: pick(['BR', 'US', 'JP', 'DE', 'EG']), language: pick(['pt-BR', 'en-GB', 'zh-Hant', 'ja']), currency: pick(CURRENCIES), script: pick(['Latn', 'Arab', 'Hani', 'Cyrl']) }[type]; return new Intl.DisplayNames(pick(LOCALES), { type, style: pick(['long', 'short', 'narrow']) }).of(code); });
make('intlCollator', 300, () => { const c = new Intl.Collator(pick(LOCALES), { numeric: pick([true, false]), sensitivity: pick(['base', 'accent', 'case', 'variant']), caseFirst: pick(['upper', 'lower', 'false']) }); return WORDS.slice().sort(c.compare).join(' '); });
make('intlSegmenter', 300, () => { const g = pick(['grapheme', 'word', 'sentence']); return [...new Intl.Segmenter(pick(LOCALES), { granularity: g }).segment(pick(TEXTS))].map((s) => s.segment + (s.isWordLike === undefined ? '' : s.isWordLike ? '+' : '-')).join('|'); });
make('intlLocale', 200, () => { const l = new Intl.Locale(pick(LOCALES)).maximize(); return [l.toString(), new Intl.Locale(pick(LOCALES)).minimize().toString(), l.script, l.region, l.getWeekInfo ? JSON.stringify(l.getWeekInfo()) : 'n/a', l.getTextInfo ? JSON.stringify(l.getTextInfo()) : 'n/a'].join(' '); });
out.intlSupported = ['calendar', 'collation', 'currency', 'numberingSystem', 'timeZone', 'unit'].map((k) => k + '=' + Intl.supportedValuesOf(k).join(','));
out.intlDefaults = [new Intl.DateTimeFormat().resolvedOptions().locale, new Intl.NumberFormat().resolvedOptions().locale, Intl.DateTimeFormat().resolvedOptions().calendar, String(Intl.getCanonicalLocales('EN-us')), String(Intl.DateTimeFormat.supportedLocalesOf(LOCALES)), String(Intl.NumberFormat.supportedLocalesOf(LOCALES))];
emit(out);
