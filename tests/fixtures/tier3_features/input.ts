// Tier 3 Features Test

// 1. Type Aliases
type ID = string;
type Score = number;

// 2. String Union -> Enum
type Status = "active" | "inactive" | "pending";

interface UserConfig {
  id: ID;
  score: Score;
  status: Status;
  // Record -> HashMap
  attributes: Record<string, string>;
}

function processFeatures(config: UserConfig) {
  // 3. for..in loop (on Record/Map)
  for (const key in config.attributes) {
    // key should be string
    // We can access value using index (if supported) or just print key
    // Note: config.attributes[key] might need support in func.rs for MemberExpr with Computed prop
    // We already have it: 
    // swc_ecma_ast::MemberProp::Computed(computed) => { ... #obj[#prop] }
    // Rust HashMap indexing with [] panics if missing. 
    // But iterating keys ensures it exists.
    // However, Rust HashMap indexing borrows.
    // Let's just print key for now to test loop structure.
    console.log("Attribute:", key);
  }

  // 4. do..while loop
  let count = 0;
  do {
    console.log("Count:", count);
    count += 1;
  } while (count < 3);

  // Test Enum usage
  if (config.status == "active") {
    console.log("User is active");
  }
}
