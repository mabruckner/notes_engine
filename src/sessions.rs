#[derive(Clone)]
struct User {
    id: i32,
    name: String
}

#[derive(Clone)]
pub struct Session {
    user: Option<User>,
    token: String
}

impl Session {
    fn get_token(&self) -> String {
        self.token.clone()
    }

    fn upgrade(&mut self, username: &str, pass: &str) -> Result<(), String> {
        unimplemented!()
    }

    fn downgrade(&mut self) -> () {
        unimplemented!()
    }
}

pub struct Sessions {
    sessions: Vec<Session>
}

impl Sessions {
    fn get_session(&self, token: String) -> Option<Session> {
        for x in self.sessions.iter() {
            if x.token == token {
                return Some(x.clone())
            }
        }
        None
    }

    fn create_session(&mut self) -> Session {
        Session {
            user: None,
            token: "asdf"
        }
    }

    fn clear_all_sessions(&mut self) -> () {
        self.sessions.clear()
    }
}
