function f(x: number): string {
  let out: string = "";
  switch (x) {
    case 1:
      out = out + "a";
      break;
    default:
      out = out + "d";
    case 2:
      out = out + "b";
      break;
  }
  return out;
}
console.log(f(2));
console.log(f(5));
