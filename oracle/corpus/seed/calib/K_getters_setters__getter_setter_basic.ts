class Temperature {
    private _celsius: number;
    constructor(celsius: number) {
        this._celsius = celsius;
    }
    get celsius(): number {
        return this._celsius;
    }
    set celsius(value: number) {
        this._celsius = value;
    }
    get fahrenheit(): number {
        return this._celsius * 1.8 + 32;
    }
}

function run(): void {
    let temp = new Temperature(100);
    console.log(temp.celsius);
    console.log(temp.fahrenheit);
    temp.celsius = 0;
    console.log(temp.celsius);
    console.log(temp.fahrenheit);
}
run();
