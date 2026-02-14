type ID = String;
type Score = f64;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde :: Serialize, serde :: Deserialize)]
enum Status {
    #[default]
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "inactive")]
    Inactive,
    #[serde(rename = "pending")]
    Pending,
}
impl PartialEq<String> for Status {
    fn eq(&self, other: &String) -> bool {
        match self {
            Status::Active => other == "active",
            Status::Inactive => other == "inactive",
            Status::Pending => other == "pending",
        }
    }
}
impl PartialEq<&str> for Status {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Status::Active => *other == "active",
            Status::Inactive => *other == "inactive",
            Status::Pending => *other == "pending",
        }
    }
}
#[derive(Default, Debug, Clone, PartialEq, serde :: Serialize, serde :: Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserConfig {
    pub id: ID,
    pub score: Score,
    pub status: Status,
    pub attributes: std::collections::HashMap<String, String>,
}
fn process_features(config: UserConfig) -> () {
    for key in config.attributes.keys().cloned() {
        println!("{} {}", String::from("Attribute:"), key);
    }
    let mut count = 0f64;
    loop {
        {
            println!("{} {}", String::from("Count:"), count);
            count += 1f64;
        }
        if !(count < 3f64) {
            break;
        }
    }
    if config.status == String::from("active") {
        println!("{}", String::from("User is active"));
    }
}
