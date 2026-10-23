class Shape {
  area(): number {
    return 0;
  }
}
class Sq extends Shape {
  s: number;
  constructor(s: number) {
    super();
    this.s = s;
  }
  area(): number {
    return this.s * this.s;
  }
}
class Ci extends Shape {
  r: number;
  constructor(r: number) {
    super();
    this.r = r;
  }
  area(): number {
    return 3 * this.r * this.r;
  }
}
const shapes: Shape[] = [new Sq(2), new Ci(1)];
shapes.forEach((s: Shape): void => {
  console.log(s.area());
});
