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
use rand::Rng;
use rocket::http::{CookieJar, Cookie};




#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(author, 
       version, 
       about = "
        file depot
        ",
       long_about = None)]
struct Args {
    /// Directory to store files
    #[clap(short, long, value_name="DIRECTORY")]
    workdir: String,

    /// App root path
    #[clap(short, long, value_name="APPROOT")]
    approot: String,

    /// PORT to listen on
    #[clap(short, long, value_name="PORT")]
    port: u16,

    /// Generate bash completion
    #[clap(long)]
    completion: bool,
}


use rocket_basicauth::BasicAuth;

fn get_port() -> u16 { 
    let clops = Args::parse();
    clops.port
}
fn get_dir() -> PathBuf {
    let clops = Args::parse();
    Path::new(&clops.workdir).to_owned()
}
fn get_approot() -> String {
    let clops = Args::parse();
    clops.approot
}
fn get_uploaded_files() -> Vec<(String,String)> {
    let clops = Args::parse();
    std::fs::read_dir(&Path::new(&clops.workdir))
        .unwrap()
        .map(|x| {
            let y = x.unwrap().file_name().into_string().unwrap();
            (format!("{}", y), format!("{}", urlencoding::encode(&y)))
        }).collect::<Vec<(String,String)>>()
}

/// Hello route with `auth` request guard, containing a `name` and `password`
#[get("/")]
fn index(auth: BasicAuth, jar: &CookieJar<'_>) -> Template {
    let mut rng = rand::thread_rng();
    let token = format!("{}", rng.gen::<f64>());
    let cookie = Cookie::new("csrf", token.clone());
    jar.add_private(cookie);    
    Template::render(
        "index", 
        context! { 
            csrf: token,
            approot : get_approot(), 
            username: auth.username,
            files : get_uploaded_files()
        }
        )
}


#[derive(FromForm)]
struct FileUp<'r> {
    contents: TempFile<'r>,
    csrf: String
}
#[post("/upload", data = "<upload>")]
async fn upload(auth: BasicAuth, jar: &CookieJar<'_>, mut upload: Form<FileUp<'_>>) -> Result<Redirect, std::io::Error> {
    let m_cookie = jar.get_private("csrf");
    if let Some(cookie) = m_cookie {
        if cookie.clone().into_owned().value() == upload.csrf {
            let mut p = get_dir();
            let mut ext = "";
            if let Some(ct) = upload.contents.content_type() { if let Some(u) = ct.extension() { ext = u.as_str() } } 
            if let Some(name) = upload.contents.name() { p = p.join(&(String::from(name) + "." + ext)); } else { p = p.join("NONAME"); }
            upload.contents.persist_to(&p).await?;
            Ok(Redirect::to(String::from("/") + &get_approot()))
        } else {
            Ok(Redirect::to(String::from("/") + &get_approot() + "/error?message=badToken"))
        }
    } else { Ok(Redirect::to(String::from("/") + &get_approot() + "/error?message=noCookie")) }

}

#[get("/download?<filename>")]
async fn download(auth: BasicAuth, filename: String) -> Option<NamedFile> {
    let f: DirEntry = std::fs::read_dir(&get_dir()).unwrap().filter(|x| x.as_ref().unwrap().file_name().into_string() == Ok(filename.clone())).next().unwrap().unwrap();
    NamedFile::open(f.path()).await.ok()
}

#[get("/error?<message>")]
async fn show_error(auth: BasicAuth, message: String) -> String {
    message
}


#[launch]
fn rocket() -> _ {
    let clops = Args::parse();
    if clops.completion {
        generate(Bash, &mut Args::into_app(), "depot", &mut io::stdout());
    }
    let config = Config {
        port: get_port(),
        limits: Limits::default().limit("file", 100.megabytes()), 
        ..Config::debug_default()
    };
    rocket::custom(config).attach(Template::fairing()).mount(String::from("/") + &get_approot(), routes![index,upload,download,show_error])
    
}
