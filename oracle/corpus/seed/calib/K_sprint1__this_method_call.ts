class Calculator {
    double(n: number): number {
        return n * 2;
    }
    quadruple(n: number): number {
        return this.double(this.double(n));
    }
}
const c: Calculator = new Calculator();
console.log(c.quadruple(5));
