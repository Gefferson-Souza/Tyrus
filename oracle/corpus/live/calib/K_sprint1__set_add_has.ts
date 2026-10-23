function run(): void {
    let ids: Set<string> = new Set();
    ids.add("alice");
    ids.add("bob");
    ids.add("alice");
    console.log(ids.has("alice"));
    console.log(ids.has("charlie"));
    console.log(ids.size);
}
run();
