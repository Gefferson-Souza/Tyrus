// Validador mínimo de JSON Schema: só as palavras-chave que os esquemas do Tyrus usam.
// Palavra-chave desconhecida lança erro, para o esquema nunca ser "aceito" sem ser aplicado.
type Json = unknown;
type Schema = Readonly<Record<string, Json>>;

const ANNOTATIONS = new Set(['$schema', '$id', 'title', 'description', '$defs']);
const KEYWORDS = new Set([
  'type', 'const', 'enum', 'pattern', 'required', 'properties',
  'additionalProperties', 'items', 'minItems', 'minLength', '$ref',
]);

export function validate(schema: Schema, value: Json): string[] {
  const errors: string[] = [];
  check(schema, schema, value, '$', errors);
  return errors;
}

function check(root: Schema, schema: Schema, value: Json, at: string, errors: string[]): void {
  for (const key of Object.keys(schema)) {
    if (!ANNOTATIONS.has(key) && !KEYWORDS.has(key)) {
      throw new Error(`palavra-chave de esquema sem suporte: ${key}`);
    }
  }
  if (typeof schema.$ref === 'string') {
    check(root, resolveRef(root, schema.$ref), value, at, errors);
    return;
  }
  if ('const' in schema && schema.const !== value) errors.push(`${at}: esperado ${JSON.stringify(schema.const)}`);
  if (Array.isArray(schema.enum) && !schema.enum.includes(value)) errors.push(`${at}: fora do enum`);
  if (typeof schema.type === 'string' && !hasType(schema.type, value)) {
    errors.push(`${at}: esperado ${schema.type}`);
    return;
  }
  if (typeof value === 'string') checkString(schema, value, at, errors);
  if (Array.isArray(value)) checkArray(root, schema, value, at, errors);
  else if (isRecord(value)) checkObject(root, schema, value, at, errors);
}

function checkString(schema: Schema, value: string, at: string, errors: string[]): void {
  if (typeof schema.pattern === 'string' && !new RegExp(schema.pattern).test(value)) {
    errors.push(`${at}: não casa com ${schema.pattern}`);
  }
  if (typeof schema.minLength === 'number' && value.length < schema.minLength) {
    errors.push(`${at}: menor que ${schema.minLength}`);
  }
}

function checkArray(root: Schema, schema: Schema, value: Json[], at: string, errors: string[]): void {
  if (typeof schema.minItems === 'number' && value.length < schema.minItems) {
    errors.push(`${at}: menos que ${schema.minItems} itens`);
  }
  if (isRecord(schema.items)) {
    value.forEach((item, index) => check(root, schema.items as Schema, item, `${at}[${index}]`, errors));
  }
}

function checkObject(root: Schema, schema: Schema, value: Schema, at: string, errors: string[]): void {
  const properties = isRecord(schema.properties) ? schema.properties : {};
  const required = Array.isArray(schema.required) ? schema.required : [];
  for (const name of required) {
    if (!(name in value)) errors.push(`${at}: falta o campo ${String(name)}`);
  }
  for (const [name, child] of Object.entries(value)) {
    const sub = properties[name];
    if (isRecord(sub)) check(root, sub, child, `${at}.${name}`, errors);
    else if (schema.additionalProperties === false) errors.push(`${at}: campo ${name} não permitido`);
  }
}

function resolveRef(root: Schema, ref: string): Schema {
  const match = /^#\/\$defs\/([A-Za-z0-9_]+)$/.exec(ref);
  const defs = isRecord(root.$defs) ? root.$defs : {};
  const target = match === null ? undefined : defs[match[1] as string];
  if (!isRecord(target)) throw new Error(`$ref sem suporte ou inexistente: ${ref}`);
  return target;
}

function isRecord(value: Json): value is Schema {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function hasType(type: string, value: Json): boolean {
  if (type === 'array') return Array.isArray(value);
  if (type === 'object') return isRecord(value);
  if (type === 'integer') return Number.isInteger(value);
  if (type === 'null') return value === null;
  return typeof value === type;
}
