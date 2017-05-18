use persistent::Read;
use user::*;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::path::PathBuf;
use glob;
use glob::Pattern;
use std::error::Error;
use iron;
use iron::prelude::*;
use iron_sessionstorage::traits::*;
pub struct AccessPath;

impl iron::typemap::Key for AccessPath {
    type Value = PathBuf;
}

pub struct Access {
    globs: Vec<Pattern>
}

impl Access {
    pub fn from_req(req: &mut Request) -> Result<Access, Box<Error>> {
        let username = req.session().get::<User>()?.map(|x| x.username).unwrap_or("default".into());
        let configpath = {
            let path = req.get::<Read<AccessPath>>()?.join(&username);
            if path.is_file() {
                path
            } else {
                // let a nonexistent default config fall through and hit open - that'll catch it.
                req.get::<Read<AccessPath>>()?.join("default")
            }
        };
        let globfile = File::open(configpath)?;
        let mut globs = Vec::new();
        for globline in BufReader::new(globfile).lines() {
            globs.push(Pattern::new(&globline?)?);
        }
        Ok(Access {
            globs: globs
        })
    }
    pub fn matches(&self, path: &str) -> bool {
        for p in &self.globs {
            if p.matches(path) {
                return true;
            }
        }
        false
    }
    pub fn enumerate(&self, base: PathBuf) -> Result<Vec<PathBuf>, Box<Error>> {
        let prefix = Pattern::escape(base.to_str().unwrap());
        let mut results = Vec::new();
        for glob in &self.globs {
            for path in glob::glob(&[&prefix, glob.as_str()].join("/"))? {
                results.push(path?.strip_prefix(&base)?.to_path_buf());
            }
        }
        results.sort();
        results.dedup();
        Ok(results)
    }
}
