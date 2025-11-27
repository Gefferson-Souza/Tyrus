export default class User {
    name: string;
    constructor(name: string) {
        this.name = name;
    }
    greet(): void {
        console.log("Hello,", this.name);
    }
}
