let m: Map<string, number> = new Map<string, number>();
m.set("alpha", 1);
m.set("bravo", 2);
m.set("charlie", 3);
m.set("delta", 4);
m.set("echo", 5);
m.forEach((v: number, k: string) => {
    console.log(k + "=" + v);
});
