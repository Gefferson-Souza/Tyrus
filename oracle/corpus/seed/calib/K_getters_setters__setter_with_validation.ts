class Percentage {
    private _value: number;
    constructor(value: number) {
        this._value = value;
    }
    get value(): number {
        return this._value;
    }
    set value(v: number) {
        if (v > 100) {
            this._value = 100;
        } else if (v < 0) {
            this._value = 0;
        } else {
            this._value = v;
        }
    }
}

function run(): void {
    let p = new Percentage(50);
    console.log(p.value);
    p.value = 200;
    console.log(p.value);
    p.value = -10;
    console.log(p.value);
}
run();
