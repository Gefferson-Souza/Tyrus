class Summer {
  total: number = 0;
  addAll(xs: number[]): void {
    xs.forEach((x: number): void => {
      this.total += x;
    });
  }
}
let s: Summer = new Summer();
s.addAll([1, 2, 3]);
console.log(s.total);
