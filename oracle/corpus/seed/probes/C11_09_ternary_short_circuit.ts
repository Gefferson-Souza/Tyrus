function sign(n: number): string {
  return n < 0 ? "neg" : n === 0 ? "zero" : "pos";
}
function t(): boolean {
  console.log("t");
  return true;
}
function f(): boolean {
  console.log("f");
  return false;
}
console.log(sign(-1), sign(0), sign(3));
const r1: boolean = f() && t();
const r2: boolean = t() || f();
console.log(r1, r2);
