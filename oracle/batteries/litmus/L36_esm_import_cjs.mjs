import fs from 'node:fs';
import { createRequire } from 'node:module';
import cjsDefault, { named } from './fixtures/cjs_dep.cjs';
const L = (s) => fs.writeSync(1, s + '\n');
const require = createRequire(import.meta.url);
L('default-' + cjsDefault.named + '-' + cjsDefault.extra);
L('named-' + named);
L('require-same-' + (require('./fixtures/cjs_dep.cjs') === cjsDefault));
L('meta-' + import.meta.url.endsWith('/L36_esm_import_cjs.mjs'));
