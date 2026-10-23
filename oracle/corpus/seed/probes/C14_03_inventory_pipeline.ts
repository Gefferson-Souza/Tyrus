interface Product {
  name: string;
  price: number;
  qty: number;
}
const products: Product[] = [
  { name: "pen", price: 2, qty: 10 },
  { name: "book", price: 15, qty: 3 },
  { name: "bag", price: 40, qty: 0 },
];
const inStock: Product[] = products.filter((p: Product): boolean => p.qty > 0);
const total: number = inStock.reduce((acc: number, p: Product): number => acc + p.price * p.qty, 0);
const names: string[] = inStock.map((p: Product): string => p.name.toUpperCase());
console.log(names.join(", "));
console.log("Total: " + total);
