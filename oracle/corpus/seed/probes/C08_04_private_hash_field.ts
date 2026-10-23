class Acc {
  #total: number = 0;
  add(n: number): void {
    this.#total = this.#total + n;
  }
  get(): number {
    return this.#total;
  }
}
const a: Acc = new Acc();
a.add(2);
a.add(3);
console.log(a.get());
