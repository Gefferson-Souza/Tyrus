use super::models::User;
use super::utils::add_and_double;
pub fn main() -> () {
    let res = add_and_double(2f64, 3f64);
    println!("{} {}", String::from("Result:"), res);
    let user = User::new(String::from("TypeRust"));
    user.greet();
    if res == 10f64 {
        println!("{}", String::from("✅ Phase 2 Project Test Passed!"));
    } else {
        println!("{}", String::from("❌ Test Failed"));
    }
}
