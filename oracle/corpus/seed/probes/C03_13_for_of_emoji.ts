const s: string = "a😀ç";
let n: number = 0;
for (const ch of s) {
  n = n + 1;
  console.log(ch);
}
console.log(n);
