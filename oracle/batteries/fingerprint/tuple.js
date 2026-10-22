'use strict';
const fs = require('fs');
const v = process.versions;
const runtime = v.bun ? 'bun' : v.deno ? 'deno' : 'node';
fs.writeSync(1, JSON.stringify({
  runtime,
  version: runtime === 'bun' ? v.bun : process.version,
  v8: v.v8 || null,
  icu: v.icu || null,
  unicode: v.unicode || null,
  tz: v.tz || null,
  cldr: v.cldr || null,
}));
