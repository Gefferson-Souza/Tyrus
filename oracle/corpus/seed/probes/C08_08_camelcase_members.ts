class Cart {
  itemCount: number;
  constructor(initialCount: number) {
    this.itemCount = initialCount;
  }
  addItems(extraCount: number): number {
    this.itemCount += extraCount;
    return this.itemCount;
  }
}
let myCart: Cart = new Cart(1);
console.log(myCart.addItems(2));
