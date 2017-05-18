use iron::prelude::*;
use user::*;
use logins::*;
use iron_sessionstorage::traits::*;
use persistent::*;
use access::Access;
use config::*;

#[derive(Serialize)]
pub struct Admin {
    pub users: Vec<(String, bool)>
}

#[derive(Serialize)]
pub struct Base<T> {
    pub user: Option<String>,
    pub is_admin: bool,
    pub page: T,
    pub current: String,
    pub accessible: Vec<String>
}

impl <T> Base<T> {
    pub fn from_req(req: &mut Request, page: T) -> Self {
        let base = req.get::<Read<BasePath>>().unwrap().to_path_buf();
        let username = req.session().get::<User>().unwrap().map(|x| x.username);
        let admin = username.clone().map(|x| req.get::<Read<Logins>>().unwrap().get_login(&x).map(|x| x.admin).unwrap_or(false)).unwrap_or(false);
        Base {
            user: username,
            is_admin: admin,
            page: page,
            current: req.url.path().join("/"),
            accessible: match Access::from_req(req) {
                Ok(access) => {
                    match access.enumerate(base) {
                        Ok(vals) => vals.into_iter().filter_map(|x| x.into_os_string().into_string().ok()).collect(),
                        Err(_) => vec![]
                    }
                },
                Err(_) => {
                    vec![]
                }
            }
        }
    }
}

#[derive(Serialize)]
pub struct Dir {
    pub children: Vec<(String, String)>
}

impl Dir {
    pub fn from_req(req: &mut Request) -> Base<Dir> {
        Self::from_base(Base::from_req(req, Dir { children: vec![] }))
    }
    fn from_base(base: Base<Self>) -> Base<Self> {
        let mut children = Vec::new();
        for x in &base.accessible {
            if x.starts_with(&base.current) {
                children.push((x[base.current.len()..].into(), x.clone()));
            }
        }
        Base {
            page: Dir { children: children },
            ..base
        }
    }
}
