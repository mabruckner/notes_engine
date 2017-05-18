extern crate iron;
extern crate handlebars;
extern crate handlebars_iron as hbs;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate iron_sessionstorage;
extern crate csv;
extern crate bcrypt;
extern crate persistent;
#[macro_use]
extern crate router;
extern crate params;
extern crate hoedown;
extern crate glob;
extern crate rand;
extern crate toml;

use iron::prelude::*;
use iron::{status, Handler};
use iron::headers::{Cookie, SetCookie};

use hbs::{HandlebarsEngine, DirectorySource, Template};
use hbs::handlebars::to_json;
#[cfg(feature = "watch")]
use hbs::Watchable;

use iron_sessionstorage::{SessionStorage, Value};
use iron_sessionstorage::backends::SignedCookieBackend;
use iron_sessionstorage::traits::*;

use rand::Rng;

use std::path::PathBuf;
use std::fs::File;
use std::sync::Arc;

mod logins;
mod admin;
mod user;
mod templates;
mod access;
mod config;

use logins::*;
use user::*;
use access::*;
use config::*;

fn raw_handler(request: &mut Request) -> IronResult<Response> {
    use std::io::Read;
    let path = request.get::<persistent::Read<BasePath>>().unwrap();
    let path = path.join(request.url.path().join("/"));
    if path.is_file() {
        println!("{:?}", path);
        Ok(Response::with((status::Ok,File::open(path).unwrap())))
    } else {
        Ok(Response::with(status::NotFound))
    }
}

fn code_handler(request: &mut Request) -> IronResult<Response> {
    use std::io::Read;
    let path = request.get::<persistent::Read<BasePath>>().unwrap();
    let path = path.join(request.url.path().join("/"));
    println!("{:?}", path);
    if path.is_file() {
        let mut buf = String::new();
        File::open(path).unwrap().read_to_string(&mut buf);
        let mut response = Response::new();
        response
            .set_mut(Template::new("code", to_json(&templates::Base::<String>::from_req(request, buf))))
            .set_mut((status::Ok, "Hello World"));
        return Ok(response)
    }
    unimplemented!();
}

fn directory_handler(request: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, Template::new("dir", to_json(&templates::Dir::from_req(request))))))
}

fn markdown_handler(request: &mut Request) -> IronResult<Response> {
    use hoedown::*;
    let path = request.get::<persistent::Read<BasePath>>().unwrap();
    let path = path.join(request.url.path().join("/"));
    println!("{:?}", path);
    if path.is_file() {
        let mark = Markdown::read_from(File::open(path).unwrap()).extensions(FENCED_CODE | MATH | MATH_EXPLICIT | SPACE_HEADERS | TABLES);
        let mut html = Html::new(renderer::html::Flags::empty(), 0);
        let content = html.render(&mark).to_str().unwrap().into();

        let mut response = Response::new();
        response
            .set_mut(Template::new("markdown", to_json(&templates::Base::<String>::from_req(request, content))))
            .set_mut((status::Ok, "Hello World"));
        return Ok(response)
    }
    unimplemented!();
}

fn typed_handler(request: &mut Request) -> IronResult<Response> {
    let path: PathBuf = request.url.path().join("/").into();
    let base = request.get::<persistent::Read<BasePath>>().unwrap();
    match path.extension().map(|x| x.to_str()).unwrap_or(None) {
        Some("md") => markdown_handler(request),
        Some("txt") => code_handler(request),
        None => {
            let dir = base.join(&path);
            println!("dir: {:?}", dir);
            println!("dir: {:?}", dir.with_file_name("HELLO_WORLD.txt"));
            let mut url: iron::url::Url = request.url.clone().into();
            if dir.join("index.md").is_file() {
                url.set_path(&path.join("index.md").into_os_string().into_string().unwrap());
                Ok(Response::with((status::SeeOther, iron::modifiers::Redirect(iron::Url::from_generic_url(url).unwrap()))))
            } else if dir.join("index.html").is_file() {
                url.set_path(&path.join("index.html").into_os_string().into_string().unwrap());
                Ok(Response::with((status::SeeOther, iron::modifiers::Redirect(iron::Url::from_generic_url(url).unwrap()))))
            } else {
                directory_handler(request)
            }
        },
        _ => {
            println!("{:?}", path);
            raw_handler(request)
        }
    }
}

struct NotesHandler {
    base: PathBuf,
}

impl Handler for NotesHandler {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        let user = request.session().get::<User>()?;
        let logins = request.get::<persistent::Read<Logins>>().unwrap();
        println!("{:?}", user);
        println!("{:?}", request.url.path());
        println!("{:?}", logins.as_ref().get_logins());
        let mut response = Response::new();
        response
            .set_mut(Template::new("markdown", to_json(&templates::Base::<String>::from_req(request, "Hello World!".into()))))
            .set_mut((status::Ok, "Hello World"));
        Ok(response)
    }
}

struct Dispatch<A:Handler, B:Handler> {
    a_path: String,
    admin: A,
    general: B
}

impl <A:Handler, B:Handler> Handler for Dispatch<A, B> {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        if request.url.path().get(0) == Some(&self.a_path.as_str()) {
            self.admin.handle(request)
        } else {
            self.general.handle(request)
        }
    }
}

struct AccessControl<T:Handler> {
    handler:T,
    path: PathBuf
}

impl <T:Handler> Handler for AccessControl<T> {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        let target = request.url.path().join("/");
        let access = Access::from_req(request).unwrap();
        if access.matches(&target) {
            self.handler.handle(request)
        } else {
            Ok(Response::with((status::NotFound, "you do not have access to the requested resource")))
        }
    }
}

#[cfg(feature="watch")]
fn handlebars() -> Arc<HandlebarsEngine> {
    let mut hbse = HandlebarsEngine::new();
    hbse.add(Box::new(DirectorySource::new("./src/templates", ".hbs")));
    hbse.reload().unwrap();
    let hbse_ref = Arc::new(hbse);
    hbse_ref.watch("./src/templates/");
    hbse_ref
}

#[cfg(not(feature="watch"))]
fn handlebars() -> HandlebarsEngine {
    let mut hbse = HandlebarsEngine::new();
    hbse.add(Box::new(DirectorySource::new("./src/templates/", ".hbs")));
    hbse.reload().unwrap();
    hbse
}

fn main() {
    use std::env::args;
    let config_filepath = args().nth(1).unwrap_or("config.toml".into());
    let config = Configuration::load(config_filepath.into()).unwrap();

    let hbse = handlebars();

    let handler = AccessControl {
        handler: typed_handler,
        path: config.access_path.clone()
    };

    let mut chain = Chain::new(Dispatch {
        a_path: "admin".into(),
        admin: admin::route_admin(),
        general: handler
    });
    chain.link(persistent::Read::<Logins>::both(Logins {
        path: config.user_path.into()
    }));
    chain.link(persistent::Read::<BasePath>::both::<PathBuf>(config.base_path.into()));
    chain.link(persistent::Read::<AccessPath>::both::<PathBuf>(config.access_path.into()));
    chain.link_after(hbse);

    let mut secret = [0; 32];
    rand::OsRng::new().unwrap().fill_bytes(&mut secret);

    chain.link_around(SessionStorage::new(SignedCookieBackend::new(secret.to_vec())));

    let server = Iron::new(chain).http(&config.serve).unwrap();
    println!("started");
}
