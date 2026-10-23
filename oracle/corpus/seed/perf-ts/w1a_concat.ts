function buildConcat(n: number): string {
    let s: string = "";
    for (let i: number = 0; i < n; i++) {
        s += "item" + i + ",";
    }
    return s;
}
const n: number = 2000000;
const text: string = buildConcat(n);
console.log(text.length);
console.log(text.indexOf("item1999999,"));
