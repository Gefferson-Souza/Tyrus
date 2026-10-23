interface Item {
    sku: number;
    label: string;
    price: number;
    tags: string[];
}
const a: Item = { sku: 3, label: "x", price: 2, tags: ["a"] };
const text: string = JSON.stringify(a);
console.log(text);
console.log(text.length);
