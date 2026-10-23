function makeText(n: number): string {
    let words: string[] = [];
    for (let i: number = 0; i < n; i++) {
        words.push("word" + (i % 1000));
    }
    return words.join(" ");
}
const n: number = 1000000;
const text: string = makeText(n);
const parts: string[] = text.split(" ");
const partCount: number = parts.length;
let hits: number = 0;
for (const p of parts) {
    const q: string = p;
    if (q.indexOf("99") >= 0) {
        hits++;
    }
}
const replaced: string = text.split("word").join("W");
const upper: string = text.toUpperCase();
console.log(partCount);
console.log(hits);
console.log(replaced.length);
console.log(upper.length);
console.log(upper.indexOf("WORD999"));
