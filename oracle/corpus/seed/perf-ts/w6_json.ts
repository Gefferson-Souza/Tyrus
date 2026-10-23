interface Item {
    sku: number;
    label: string;
    price: number;
    tags: string[];
}
function makeItems(n: number): Item[] {
    const out: Item[] = [];
    for (let i: number = 0; i < n; i++) {
        out.push({ sku: i, label: "item" + i, price: i * 1.5 + 0.25, tags: ["a", "b"] });
    }
    return out;
}
const n: number = 50000;
const rounds: number = 5;
let total: number = 0;
let size: number = 0;
for (let r: number = 0; r < rounds; r++) {
    const items: Item[] = makeItems(n);
    const text: string = JSON.stringify(items);
    const back: Item[] = JSON.parse(text);
    size = text.length;
    for (const it of back) {
        total += it.price;
    }
}
console.log(size);
console.log(total);
