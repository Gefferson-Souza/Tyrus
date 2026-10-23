let s: Set<string> = new Set();
s.add("c");
s.add("a");
s.add("b");
s.add("a");
s.add("z");
s.add("m");
s.add("q");
s.add("k");
s.add("x");
s.add("d");
s.add("p");
s.forEach((v: string): void => {
  console.log(v);
});
