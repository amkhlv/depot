#[macro_use] extern crate rocket;
use rocket::config::Config;
use clap::{Parser,IntoApp};
use clap_complete::{generate, shells::Bash};
use std::io;
use std::path::{Path,PathBuf};
use rocket_dyn_templates::{Template,context};
use rocket::response::Redirect;
use rocket::fs::{TempFile,NamedFile};
use std::fs::DirEntry;
use rocket::form::Form;
use rocket::data::{ToByteUnit, Limits};
use rocket::serde::Deserialize;
use rand::Rng;
use rocket::http::{CookieJar, Cookie};
use rocket::State;

use rocket_basicauth::BasicAuth;

struct Parameters {
    approot: String,
    workdir: String 
}

fn get_uploaded_files(workdir: &Path) -> Vec<(String,String)> {
    std::fs::read_dir(workdir)
        .unwrap()
        .map(|x| {
            let y = x.unwrap().file_name().into_string().unwrap();
            (format!("{}", y), format!("{}", urlencoding::encode(&y)))
        }).collect::<Vec<(String,String)>>()
}

/// Hello route with `auth` request guard, containing a `name` and `password`
#[get("/")]
fn index(auth: BasicAuth, jar: &CookieJar<'_>, state: &State<Parameters>) -> Template {
    let mut rng = rand::thread_rng();
    let token = format!("{}", rng.gen::<f64>());
    let cookie = Cookie::new("csrf", token.clone());
    jar.add_private(cookie);    
    Template::render(
        "index", 
        context! { 
            csrf: token,
            approot : state.approot.clone(), 
            username: auth.username,
            files : get_uploaded_files(Path::new(&state.workdir))
        }
        )
}


#[derive(FromForm)]
struct FileUp<'r> {
    contents: TempFile<'r>,
    csrf: String
}
#[post("/upload", data = "<upload>")]
async fn upload(auth: BasicAuth, jar: &CookieJar<'_>, mut upload: Form<FileUp<'_>>, state: &State<Parameters>) -> Result<Redirect, std::io::Error> {
    let m_cookie = jar.get_private("csrf");
    if let Some(cookie) = m_cookie {
        if cookie.clone().into_owned().value() == upload.csrf {
            let mut p = PathBuf::new();
            p.push(Path::new(&state.workdir));
            let mut ext = "";
            if let Some(ct) = upload.contents.content_type() { if let Some(u) = ct.extension() { ext = u.as_str() } } 
            if let Some(name) = upload.contents.name() { p = p.join(&(String::from(name) + "." + ext)); } else { p = p.join("NONAME"); }
            upload.contents.persist_to(&p).await?;
            Ok(Redirect::to(String::from("/") + &state.approot))
        } else {
            Ok(Redirect::to(String::from("/") + &state.approot + "/error?message=badToken"))
        }
    } else { Ok(Redirect::to(String::from("/") + &state.approot + "/error?message=noCookie")) }

}

#[get("/download?<filename>")]
async fn download(auth: BasicAuth, filename: String, state: &State<Parameters>) -> Option<NamedFile> {
    let f: DirEntry = std::fs::read_dir(&state.workdir).unwrap().filter(|x| x.as_ref().unwrap().file_name().into_string() == Ok(filename.clone())).next().unwrap().unwrap();
    NamedFile::open(f.path()).await.ok()
}

#[get("/error?<message>")]
async fn show_error(auth: BasicAuth, message: String) -> String {
    message
}

#[launch]
fn rocket() -> _ {
    let rocket = rocket::build();
    let figment = rocket.figment();
    let approot: String = figment.extract_inner("approot").expect("approot");
    let workdir: String = {
        use std::env;
        if let Ok(conf_path) = env::var("ROCKET_CONFIG") {
            println!("ROCKET_CONFIG={}",conf_path);
            Path::new(&conf_path).parent().unwrap().join("files").to_str().unwrap().to_owned()
        } else { panic!("ROCKET_CONFIG environment variable should point to the location of the configuration .toml file") }
    };
    println!("workdir={}",workdir);
    println!("approot={}",approot);
    rocket
        .attach(Template::fairing())
        .mount(String::from("/") + &approot, routes![index,upload,download,show_error])
        .manage(Parameters { approot, workdir })
    
}
