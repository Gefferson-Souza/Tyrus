let m: Map<string, number> = new Map();
m.set("zeta", 1);
m.set("alpha", 2);
m.set("mid", 3);
m.set("beta", 4);
m.set("omega", 5);
m.set("gamma", 6);
m.set("delta", 7);
m.set("kappa", 8);
m.set("sigma", 9);
m.set("theta", 10);
m.forEach((v: number, k: string): void => {
  console.log(k);
});
