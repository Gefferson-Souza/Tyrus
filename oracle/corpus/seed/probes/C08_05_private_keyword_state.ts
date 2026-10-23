class Acc {
  private total: number = 0;
  add(n: number): void {
    this.total += n;
  }
  getTotal(): number {
    return this.total;
  }
}
let a: Acc = new Acc();
a.add(2);
a.add(3);
console.log(a.getTotal());
