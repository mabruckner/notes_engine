use csv;
use bcrypt;
use iron;

use std::collections::HashMap;
use std::path::PathBuf;
use std::fs::OpenOptions;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Login {
    pub name: String,
    pub admin: bool,
    pub hashword: String
}

impl Login {
    pub fn verify(&self, pass: &str) -> bool {
        bcrypt::verify(pass, &self.hashword).unwrap()
    }
    pub fn new(name: &str, admin: bool, password: &str) -> Login {
        Login {
            name: String::from(name),
            admin: admin,
            hashword: bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap()
        }
    }
}

pub struct Logins {
    pub path: PathBuf
}

impl Logins {
    pub fn login_map(&self) -> HashMap<String, Login> {
        let mut map = if let Ok(mut reader) = csv::Reader::from_file(&self.path) {
            let mut map = HashMap::new();
            for row in reader.decode() {
                let (name, admin, hashword): (String, bool, String) = row.unwrap();
                map.insert(name.clone(),
                    Login {
                        name: name,
                        admin: admin,
                        hashword: hashword
                    });
            }
            map
        } else {
            HashMap::new()
        };
        if map.len() == 0 {
            let admin = Login::new("admin", true, "password");
            self.set_login(&admin);
            map.insert("admin".into(), admin);
        }
        map
    }
    pub fn get_logins(&self) -> Vec<Login> {
        self.login_map().values().cloned().collect()
    }
    pub fn get_login(&self, name: &str) -> Option<Login> {
        self.login_map().get(name).cloned()
    }
    pub fn set_login(&self, login: &Login) -> () {
        let mut file = OpenOptions::new().append(true).create(true).open(&self.path).expect("unable to open user file");
        let mut writer = csv::Writer::from_writer(file);
        writer.encode((&login.name, &login.admin, &login.hashword)).expect("unable to write record");
    }
}

impl iron::typemap::Key for Logins {
    type Value = Logins;
}
