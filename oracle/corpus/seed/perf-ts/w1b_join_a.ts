function buildJoin(n: number): string {
    let parts: string[] = [];
    for (let i: number = 0; i < n; i++) {
        parts.push("item" + i);
    }
    return parts.join(",");
}
const n: number = 2000000;
const text: string = buildJoin(n);
console.log(text.length);
console.log(text.indexOf("item1999999"));
