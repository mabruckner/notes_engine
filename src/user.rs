use iron_sessionstorage::Value;

#[derive(Debug)]
pub struct User {
    pub username: String
}

impl Value for User {
    fn get_key() -> &'static str {
        "user"
    }
    fn into_raw(self) -> String {
        self.username
    }
    fn from_raw(value: String) -> Option<Self> {
        if value.is_empty() {
            None
        } else {
            Some(User {
                username: value
            })
        }
    }
}

