'use strict';
const { unit } = require('./prng.js');
const emit = require('./emit.js');
const zones = Intl.supportedValuesOf('timeZone');
const fixed = [Date.UTC(1900, 0, 1), Date.UTC(1970, 0, 1), Date.UTC(1999, 11, 31), Date.UTC(2007, 6, 1)];
const stamps = fixed.concat(Array.from({ length: 60 }, () => Math.floor(Date.UTC(1950, 0, 1) + unit() * (Date.UTC(2045, 0, 1) - Date.UTC(1950, 0, 1)))));
const offsetOf = (zone) => {
  const fmt = new Intl.DateTimeFormat('en-US', { timeZone: zone, timeZoneName: 'longOffset', hour: 'numeric' });
  return stamps.map((t) => fmt.formatToParts(t).find((p) => p.type === 'timeZoneName').value.replace('GMT', '') || 'Z').join(' ');
};
const nameOf = (zone) => ['long', 'short', 'longGeneric', 'shortGeneric'].map((style) => ['en-US', 'pt-BR'].map((loc) => new Intl.DateTimeFormat(loc, { timeZone: zone, timeZoneName: style }).formatToParts(Date.UTC(2024, 0, 15)).find((p) => p.type === 'timeZoneName').value + '/' + new Intl.DateTimeFormat(loc, { timeZone: zone, timeZoneName: style }).formatToParts(Date.UTC(2024, 6, 15)).find((p) => p.type === 'timeZoneName').value).join(';')).join(';');
const out = {};
out.tzOffsets = zones.map((z) => z + ' ' + offsetOf(z));
out.tzNames = zones.map((z) => z + ' ' + nameOf(z));
out.tzFuture = zones.map((z) => { const fmt = new Intl.DateTimeFormat('en-US', { timeZone: z, timeZoneName: 'shortOffset', hour: 'numeric' }); return z + ' ' + Array.from({ length: 24 }, (_, i) => fmt.formatToParts(Date.UTC(2030 + (i >> 2), (i % 4) * 3, 1)).find((p) => p.type === 'timeZoneName').value).join(' '); });
out.tzIdentity = ['Asia/Calcutta', 'Asia/Kolkata', 'Europe/Kiev', 'Europe/Kyiv', 'America/Buenos_Aires', 'US/Pacific', 'GMT', 'Etc/GMT+3', 'Brazil/East', 'utc', 'Asia/Katmandu'].map((z) => { try { return z + '=' + new Intl.DateTimeFormat('en-US', { timeZone: z }).resolvedOptions().timeZone; } catch (e) { return z + '=THROW ' + e.constructor.name + ': ' + e.message; } });
out.tzDateParse = Array.from({ length: 500 }, () => { const t = Math.floor(Date.UTC(1970, 0, 1) + unit() * 3e12); const d = new Date(t); return [Date.parse(d.toISOString().replace('Z', '+03:00')), Date.parse(d.toUTCString()), Date.parse('2024-03-10T02:30:00')].join(','); });
emit(out);
