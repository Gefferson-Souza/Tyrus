function run(): void {
    let m: Map<string, string> = new Map();
    m.set("a", "hello");
    m.set("b", "world");
    m.delete("a");
    console.log(m.has("a"));
    console.log(m.size);
}
run();
