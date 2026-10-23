function run(): void {
    let cache: Map<string, number> = new Map();
    cache.set("alice", 42);
    cache.set("bob", 99);
    console.log(cache.has("alice"));
    console.log(cache.has("charlie"));
    console.log(cache.size);
}
run();
