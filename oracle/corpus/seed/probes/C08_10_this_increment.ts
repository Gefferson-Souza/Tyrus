class Counter {
  count: number = 0;
  inc(): void {
    this.count++;
  }
  get(): number {
    return this.count;
  }
}
let c: Counter = new Counter();
c.inc();
c.inc();
console.log(c.get());
