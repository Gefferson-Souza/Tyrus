function combine(): string {
    const arr1: string[] = ["a", "b", "c"];
    const arr2: string[] = ["d", "e", "f"];
    const combined: string[] = [...arr1, ...arr2];
    return combined.join(", ");
}
console.log(combine());
