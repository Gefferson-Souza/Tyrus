let m: Map<string, number> = new Map();
m.set("b", 2);
m.set("a", 1);
for (const [k, v] of m) {
  console.log(k, v);
}
