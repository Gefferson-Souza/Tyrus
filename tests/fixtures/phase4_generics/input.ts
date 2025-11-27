export interface Box<T> {
    value: T;
}

export class Wrapper<T> {
    value: T;
    constructor(value: T) {
        this.value = value;
    }
    getValue(): T {
        return this.value;
    }
}

export function identity<T>(arg: T): T {
    return arg;
}

export function process(): void {
    let b: Box<number> = { value: 10 };
    let w = new Wrapper<string>("hello");
    let i = identity<boolean>(true);
}
