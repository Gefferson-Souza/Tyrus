let m: Map<string, number> = new Map();
m.set("a", 1);
console.log(m.get("a") ?? -1);
console.log(m.get("zzz") ?? -1);
