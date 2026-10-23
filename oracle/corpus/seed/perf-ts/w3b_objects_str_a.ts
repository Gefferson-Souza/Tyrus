interface Person {
    name: string;
    age: number;
    email: string;
}
function makePeople(n: number): Person[] {
    let out: Person[] = [];
    for (let i: number = 0; i < n; i++) {
        const p: Person = { name: "user" + i, age: i % 90, email: "u" + i + "@example.com" };
        out.push(p);
    }
    return out;
}
function sumAges(people: Person[]): number {
    let sum: number = 0;
    for (const p of people) {
        sum += p.age;
    }
    return sum;
}
const n: number = 1500000;
const people: Person[] = makePeople(n);
console.log(people.length);
console.log(sumAges(people));
