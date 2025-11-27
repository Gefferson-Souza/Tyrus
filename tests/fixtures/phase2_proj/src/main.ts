import { addAndDouble } from './utils/index';
import User from './models';

export function main(): void {
    let res = addAndDouble(2, 3);
    console.log("Result:", res);

    let user = new User("TypeRust");
    user.greet();

    if (res == 10) {
        console.log("✅ Phase 2 Project Test Passed!");
    } else {
        console.log("❌ Test Failed");
    }
}

main();
