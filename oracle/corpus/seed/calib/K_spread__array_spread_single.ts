function copy(): string {
    const original: string[] = ["x", "y", "z"];
    const clone: string[] = [...original];
    return clone.join(", ");
}
console.log(copy());
