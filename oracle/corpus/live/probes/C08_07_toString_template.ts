class Money {
  cents: number;
  constructor(cents: number) {
    this.cents = cents;
  }
  toString(): string {
    return "$" + this.cents / 100;
  }
}
const m: Money = new Money(250);
console.log(`${m}`);
