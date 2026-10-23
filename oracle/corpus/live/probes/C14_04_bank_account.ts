class Account {
  private balance: number;
  constructor(initial: number) {
    this.balance = initial;
  }
  deposit(amount: number): void {
    if (amount <= 0) {
      throw new Error("invalid amount");
    }
    this.balance += amount;
  }
  withdraw(amount: number): boolean {
    if (amount > this.balance) {
      return false;
    }
    this.balance -= amount;
    return true;
  }
  getBalance(): number {
    return this.balance;
  }
}
const acc: Account = new Account(100);
acc.deposit(50);
console.log(acc.withdraw(30));
console.log(acc.withdraw(500));
console.log(`Balance: ${acc.getBalance()}`);
