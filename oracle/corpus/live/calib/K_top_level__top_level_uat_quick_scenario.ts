interface User {
    id: number;
    name: string;
    active: boolean;
}

function isAdult(age: number): boolean {
    return age >= 18;
}

const alice: User = {
    id: 1,
    name: "Alice",
    active: true,
};

console.log(alice.name);
console.log("isAdult(20):", isAdult(20));
