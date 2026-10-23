let m: Map<number, string> = new Map();
m.set(1, "one");
m.set(2, "two");
console.log(m.get(2) ?? "none");
console.log(m.size);
