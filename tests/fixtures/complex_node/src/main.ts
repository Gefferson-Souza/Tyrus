import { UserProcessor } from './services/user-processor';

async function main() {
    const proc = new UserProcessor();
    console.log(await proc.process("123"));
}
