class Counter {
  static count: number = 0;
  constructor() {
    Counter.count = Counter.count + 1;
  }
}
const a: Counter = new Counter();
const b: Counter = new Counter();
console.log(Counter.count);
