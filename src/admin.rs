use router::Router;
use params::{Params, Value};
use persistent::Read;


use iron::prelude::*;
use logins::*;
use user::*;
use iron::status;
use iron;
use iron::modifier::Modifier;

use iron_sessionstorage::traits::*;
use hbs::Template;
use hbs::handlebars::to_json;

use templates;

#[derive(Serialize)]
struct AdminPage {
    logged_in: bool,
    is_admin: bool,
    user: Option<String>
}

fn go_back(req: &Request) -> IronResult<Response> {
    let mut response = Response::with((status::SeeOther, ""));
    iron::modifiers::Redirect(url_for!(req, "index")).modify(&mut response);
    Ok(response)
}

fn is_admin(req: &mut Request) -> IronResult<bool> {
    let name = req.session().get::<User>()?.unwrap().username;
    let logins = req.get::<Read<Logins>>().unwrap();
    Ok(logins.get_login(&name).unwrap_or(Login { name: "".into(), admin: false, hashword: "".into()}).admin)
}

fn promote_handler(req: &mut Request) -> IronResult<Response> {
    if !is_admin(req)? {
        return Ok(Response::with((status::Unauthorized, "current user is not authorized to perform this action")));
    }
    if let Some(Value::String(username)) = req.get_ref::<Params>().unwrap().find(&["username"]).cloned() {
        let logins = req.get::<Read<Logins>>().unwrap();
        let mut target = logins.get_login(&username).unwrap();
        target.admin = true;
        logins.set_login(&target);
    }
    go_back(req)
}

fn create_handler(req: &mut Request) -> IronResult<Response> {
    if !is_admin(req)? {
        return Ok(Response::with((status::Unauthorized, "current user is not authorized to perform this action")));
    }

    if let (Some(Value::String(username)), Some(Value::String(password))) = {
        let params = req.get_ref::<Params>().unwrap();
        (params.find(&["username"]).cloned(), params.find(&["password"]).cloned())
    } {
        let logins = req.get::<Read<Logins>>().unwrap();
        if logins.get_login(&username).is_none() {
            logins.set_login(&Login::new(&username, false, &password));
        }
    }
    go_back(req)
}

fn password_handler(req: &mut Request) -> IronResult<Response> {
    if let (Some(Value::String(old_password)), Some(Value::String(new_password))) = {
        let params = req.get_ref::<Params>().unwrap();
        (params.find(&["oldpassword"]).cloned(), params.find(&["newpassword"]).cloned())
    } {
        let username = req.session().get::<User>()?.unwrap().username;
        let logins = req.get::<Read<Logins>>().unwrap();
        let mut login = logins.get_login(&username).unwrap();
        if login.verify(&old_password) {
            logins.set_login(&Login::new(&login.name, login.admin, &new_password));
        }
    }
    go_back(req)
}

fn logout_handler(req: &mut Request) -> IronResult<Response> {
    req.session().clear();
    go_back(req)
}

fn login_handler(req: &mut Request) -> IronResult<Response> {
    if let Some(username) = {
        match {
            let params = req.get_ref::<Params>().unwrap();
            (params.find(&["username"]).cloned(), params.find(&["password"]).cloned())
        } {
            (Some(Value::String(username)), Some(Value::String(password))) => {
                let logins = req.get::<Read<Logins>>().unwrap();
                if let Some(login) = logins.get_login(&username) {
                    if login.verify(&password) {
                        Some(username)
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            _ => None
        }
    } {
        req.session().set::<User>(User {
            username: username.clone()
        })?;
    }
    go_back(req)
}

pub fn index_handler(req: &mut Request) -> IronResult<Response> {
    let mut response = Response::new();
    let user = req.session().get::<User>()?;
    let page = {
        let logins = req.get::<Read<Logins>>().unwrap().get_logins();
        templates::Admin {
            users: logins.into_iter().map(|x| (x.name, x.admin)).collect()
        }
    };
    response
        .set_mut(Template::new("admin", to_json(&templates::Base::from_req(req, page))))
        .set_mut((status::Ok, ""));
    Ok(response)
}

pub fn route_admin() -> Router {
    let mut route = Router::new();
    route
        .post("admin/login", login_handler, "login")
        .post("admin/logout", logout_handler, "logout")
        .post("admin/promote", promote_handler, "promote")
        .post("admin/password", password_handler, "password")
        .post("admin/create", create_handler, "create")
        .get("admin", index_handler, "index");
    route
}
