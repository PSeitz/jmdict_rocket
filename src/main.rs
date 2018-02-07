#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
use rocket::response::NamedFile;
use std::io;
use std::path::{Path, PathBuf};
// #[get("/")]
// fn index() -> String {
//     format!("Hell rooot")
// }

#[get("/?<q>")]
fn search(q: Option<&str>) -> String {

	let search_term = q.as_ref().map(|le| &le[2..]).unwrap();
	search_term.to_string()
    // format!("Hello {}", )
}

fn main() {
    rocket::ignite().mount("/", routes![index, search, files]).launch();
}

#[get("/")]
fn index() -> io::Result<NamedFile> {
    NamedFile::open("static/index.html")
}

#[get("/<file..>")]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}
