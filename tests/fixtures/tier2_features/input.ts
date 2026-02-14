interface User {
  id: number;
  name?: string;
  config?: Config;
  tags?: string[];
}

interface Config {
  theme?: string;
  retries?: number;
}

function processUser(user: User): string {
  // Optional Chaining
  // If strict null checks are on, and consistent with Rust Option:
  // user.config is Option<Config>
  // user.config?.theme
  // If we use .map(), we get Option<Option<String>> if theme is optional.
  // If we use .flatten(), it handles double option. But fails if single option.
  // Let's see what happens.
  const theme = user.config?.theme;

  // Nullish Coalescing
  // retries is number | undefined.
  // user.config?.retries -> Option<Option<f64>>?
  // ?? 3 -> .unwrap_or(3.0)
  const retries = user.config?.retries ?? 3;

  // Parenthesized Expression
  const calc = (1 + 2) * 3;

  // Destructuring (Object)
  // user is simple ident. name is optional.
  const { id, name = "Anonymous" } = user;

  // Destructuring (Array)
  const list = ["a", "b", "c"];
  const [first, second] = list;

  return `User ${id} (${name}): Theme ${theme ?? "default"}, Retries ${retries}, Calc ${calc}, List ${first}-${second}`;
}
