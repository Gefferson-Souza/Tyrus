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
    let theme = user.config.as_ref().map(|v| v.theme.clone()).flatten();
    let retries = user
        .config
        .as_ref()
        .map(|v| v.retries.clone())
        .flatten()
        .unwrap_or(3f64);
    let calc = 1f64 + 2f64 * 3f64;
    let __destruct_val = user;
    let id = __destruct_val.id.clone();
    let name = __destruct_val
        .name
        .clone()
        .unwrap_or(String::from("Anonymous"));
    let list = vec![String::from("a"), String::from("b"), String::from("c")];
    let __destruct_val = list;
    let first = __destruct_val[0usize].clone();
    let second = __destruct_val[1usize].clone();
    return format!(
        "User {} ({}): Theme {}, Retries {}, Calc {}, List {}-{}",
        id,
        name,
        theme.unwrap_or(String::from("default")),
        retries,
        calc,
        first,
        second
    );
}
