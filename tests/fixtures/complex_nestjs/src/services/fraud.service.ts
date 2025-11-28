// A dependency service
export class FraudService {
    check(account: string): boolean {
        // Logic test
        return account.includes("SAFE");
    }
}
