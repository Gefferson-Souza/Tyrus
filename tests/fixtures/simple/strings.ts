const s = "Hello-World-Rust";
const parts = s.split("-");
const joined = parts.join(" ");

const replaced = s.replace("-", ":");

console.log(JSON.stringify({
  original: s,
  parts: parts,
  joined: joined,
  replaced: replaced // Only replaces first occurrence in basic TS/JS without global flag
}));
