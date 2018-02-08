#![feature(plugin, decl_macro, custom_derive)]
#![feature(plugin)]
#![plugin(rocket_codegen)]
#![recursion_limit="128"]

extern crate rocket;
extern crate rocket_contrib;
extern crate wana_kana;

#[macro_use]
extern crate measure_time;

extern crate reqwest;
use std::io;
use std::path::{Path, PathBuf};

#[macro_use]
extern crate serde_json;
#[macro_use] extern crate serde_derive;

use rocket::response::NamedFile;
use rocket::response::Redirect;
use rocket_contrib::Template;

use wana_kana::to_romaji::*;
use wana_kana::to_kana::*;
use wana_kana::to_hiragana::*;
use wana_kana::is_kana::*;
use wana_kana::is_kanji::*;
use wana_kana::is_romaji::*;

use wana_kana::Options;

fn query(term: &str, path: &str, levenshtein:u32, starts_with:bool) -> serde_json::Value{
    json!({"terms": [term], "path": path, "levenshtein_distance": levenshtein, "starts_with": starts_with })
}
fn boost(path: &str, boost_fun: &str, param:u32) -> serde_json::Value{
    json!({
        "path": path,
        "boost_fun": boost_fun,
        "param": param
    })
}

fn build_request(term: &str) -> serde_json::Value {
    let mut term = term.to_string();
    if term.starts_with("to ") {
        term = term[3..].to_string();
    }
    term = term.to_lowercase().trim().to_string();
    let mut ors = vec![];

    let is_original_kana_or_kanji = is_kana(&term) || is_kanji(&term);

    if contains_kanji(&term) {
        ors.push(json!({
            "search": query(&term,"kanji[].text" as &str, 0, true),
            "boost": [
                boost("commonness", "Log10", 1),
                boost("kanji[].commonness", "Log10", 1)
            ]
        }));
    }
    if is_kana(&to_kana(&term)){
        ors.push(
            json!({
                "search": query(&to_kana(&term),"kana[].text", 0, false),
                "boost": [
                    boost("commonness", "Log10", 1),
                    boost("kana[].commonness", "Log10", 1)
                ]
            }));
    }

    if is_romaji(&to_romaji(&term)) || !is_original_kana_or_kanji {
        let levenshtein = if !is_original_kana_or_kanji { 1 }else{ 0 };
        let queryString2 = to_romaji(&term);
        ors.push(
            json!({
                "search": query(&queryString2,"meanings.ger[].text", levenshtein, false),
                "boost": [
                    boost("commonness", "Log10", 1),
                    {
                        "path":"meanings.ger[].rank",
                        "expression": "10 / $SCORE"
                    }
                ]
            }));

        ors.push(
            json!({
                "search": query(&term,"meanings.eng[]", levenshtein, false),
                "boost": [boost("commonness", "Log10", 1)]
            }));
    }

    println!("query \n {}", json!({"or":ors, "top": 10, "skip": 0 }));

    json!({
        "or":ors,
        "top": 10,
        "skip": 0
    })


    // json
}

#[derive(FromForm)]
struct QueryParams {
    q: Option<String>,
    skip: Option<u32>
}


// fn tera(name: String) -> Template {
#[get("/?<params>")]
fn search(params: QueryParams) -> Template {

    if let Some(search_term) = params.q {

        println!("MAH TERM {:?}", search_term);
        let client = reqwest::Client::new();
        let mut res = {
            print_time!("REQUEST");
            client.post("https://ultimatejapanese.de/db/jmdict/search")
            .json(&build_request(&search_term))
            .send().unwrap()
        };
        println!("RES {:?}", res);

        let resp: serde_json::Value = res.json().unwrap();
        // println!("body = {}", resp);
        Template::render("result", &resp)
    } else {
        Template::render("result", json!({}))
    }
}

// #[get("/?<q>")]
// fn search(q: Option<&str>) -> String {

// 	let search_term = q.as_ref().map(|le| &le[2..]).unwrap();
// 	search_term.to_string()
// }

fn main() {
    rocket::ignite()
        .mount("/", routes![index, search, files])
        .attach(Template::fairing())
        .launch();
}

#[get("/")]
fn index() -> io::Result<NamedFile> {
    NamedFile::open("static/index.html")
}

#[get("/<file..>")]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}
