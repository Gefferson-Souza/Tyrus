interface Person {
    name: string;
    age: number;
    email: string;
}
function makePeople(n: number): Person[] {
    const out: Person[] = [];
    for (let i: number = 0; i < n; i++) {
        out.push({ name: "user" + i, age: i % 90, email: "u" + i + "@example.com" });
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
const n: number = 1000000;
const people: Person[] = makePeople(n);
console.log(sumAges(people));
console.log(people[n - 1].email);
