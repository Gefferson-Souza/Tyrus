#[derive(Default, Debug, Clone, PartialEq, serde :: Serialize, serde :: Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: f64,
    pub name: Option<String>,
    pub config: Option<Config>,
    pub tags: Option<Vec<String>>,
}
#[derive(Default, Debug, Clone, PartialEq, serde :: Serialize, serde :: Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub theme: Option<String>,
    pub retries: Option<f64>,
}
fn process_user(user: User) -> String {
    let theme = todo!();
    let retries = todo!("Unsupported binary op: {}", "\"??\"");
    let calc = 1f64 + 2f64 * 3f64;
    let list = vec![String::from("a"), String::from("b"), String::from("c")];
    return todo!();
}
