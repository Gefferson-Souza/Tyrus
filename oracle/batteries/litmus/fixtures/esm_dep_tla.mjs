import fs from 'node:fs';
fs.writeSync(1, 'eval-esm-tla-start\n');
await null;
fs.writeSync(1, 'eval-esm-tla-end\n');
export const value = 'tla';
