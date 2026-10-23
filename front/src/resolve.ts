import { createRequire } from 'node:module';

import type { TsApi } from './ts-api.ts';

export const MISSING_TYPESCRIPT_CODE = 'TYRUS-E7001';

export class MissingTypescriptError extends Error {
  readonly root: string;

  constructor(root: string) {
    super(
      `${MISSING_TYPESCRIPT_CODE}: o projeto em ${root} não tem \`typescript\` local; ` +
        'instale a versão do projeto (o Tyrus usa o TypeScript do próprio projeto).',
    );
    this.name = 'MissingTypescriptError';
    this.root = root;
  }
}

const sidecarRequire = createRequire(import.meta.url);

// `paths` força a busca a partir do projeto: o sidecar nunca usa um TypeScript próprio.
export function resolveProjectTypescript(root: string): TsApi {
  let resolved: string;
  try {
    resolved = sidecarRequire.resolve('typescript', { paths: [root] });
  } catch {
    throw new MissingTypescriptError(root);
  }
  return sidecarRequire(resolved) as TsApi;
}
