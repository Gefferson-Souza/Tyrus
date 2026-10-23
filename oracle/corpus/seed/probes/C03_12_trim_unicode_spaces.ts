const s: string = "\u00a0\u2003 hi \u3000\n";
console.log("[" + s.trim() + "]");
const t: string = "\uFEFFhi\uFEFF";
console.log("[" + t.trim() + "]");
