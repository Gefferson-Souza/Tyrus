class Calculator {
  value: number;
  history: number[];

  constructor() {
    this.value = 0;
    this.history = [];
  }

  add(n: number) {
    this.value = this.value + n;
    this.history.push(this.value);
  }

  sub(n: number) {
    this.value = this.value - n;
    this.history.push(this.value);
  }

  reset() {
    this.value = 0;
    // Re-assign array
    this.history = [];
  }
}

const calc = new Calculator();
calc.add(10);
calc.sub(5);
calc.add(20);

console.log(JSON.stringify({
  finalValue: calc.value,
  history: calc.history
}));
