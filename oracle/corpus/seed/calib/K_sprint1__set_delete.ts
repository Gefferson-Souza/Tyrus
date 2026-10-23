function run(): void {
    let s: Set<string> = new Set();
    s.add("x");
    s.add("y");
    s.delete("x");
    console.log(s.has("x"));
    console.log(s.size);
}
run();
