#![feature(plugin, decl_macro, custom_derive)]
#![feature(plugin)]
#![plugin(rocket_codegen)]
#![recursion_limit="128"]

extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate wana_kana;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate measure_time;

extern crate reqwest;
use std::path::{Path, PathBuf};

#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use rocket_contrib::{Json};
use rocket::response::NamedFile;
// use rocket::response::Redirect;
use rocket_contrib::Template;

use wana_kana::to_romaji::*;
use wana_kana::to_kana::*;
// use wana_kana::to_hiragana::*;
use wana_kana::is_kana::*;
use wana_kana::is_kanji::*;
use wana_kana::is_romaji::*;

// use wana_kana::Options;

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

fn build_search_request(term: &str) -> serde_json::Value {
    let mut term = term.to_string();
    if term.starts_with("to ") {
        term = term[3..].to_string();
    }
    term = term.to_lowercase().trim().to_string();
    let mut ors = vec![];

    let is_original_kana_or_kanji = is_kana(&term) || is_kanji(&term);

    if contains_kanji(&term) {
        ors.push(json!({
            "search": query(&term,"kanji[].text", 0, true),
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
        let query_string = to_romaji(&term);
        ors.push(
            json!({
                "search": query(&query_string,"meanings.ger[].text", levenshtein, false),
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

}


fn build_suggest_request(term: &str) -> serde_json::Value {
    let term = term.to_lowercase().trim().to_string();
    let mut suggests = vec![];

    let is_original_kana_or_kanji = is_kana(&term) || is_kanji(&term);

    if contains_kanji(&term) {
        suggests.push(query(&term,"kanji[].text", 0, true));
    }
    if is_kana(&to_kana(&term)){  //TODO maybe split into hiragana katakana
        suggests.push(query(&to_kana(&term),"kana[].text", 0, true));
    }

    if is_romaji(&to_romaji(&term)) || !is_original_kana_or_kanji {
        let levenshtein = if to_romaji(&term).chars().count() > 3 { 1 }else{ 0 };
        let query_string = to_romaji(&term);
        let mut ger = query(&query_string,"meanings.ger[].text", levenshtein, true);
        ger["token_value"] = json!({"path":"meanings.ger[].text.textindex.tokenValues",
            "boost_fun":"Linear",
            "param":1});
        suggests.push(ger);

        let mut eng = query(&query_string,"meanings.eng[]", levenshtein, true);
        eng["token_value"] = json!({"path":"meanings.eng[].textindex.tokenValues",
            "boost_fun":"Linear",
            "param":1});
        suggests.push(eng);

        suggests.push(query(&query_string,"kana[].romaji", levenshtein, true));

    }

    println!("query \n {}", json!({"suggest":suggests, "top": 10, "skip": 0 }));

    json!({
        "suggest":suggests,
        "top": 5,
        "skip": 0
    })

}

#[get("/suggest?<params>")]
fn suggest(params: QueryParams) -> Json {

    if let Some(search_term) = params.q {
        println!("Term {:?}", search_term);
        let mut res = {
            let request = build_suggest_request(&search_term);
            print_time!("REQUEST");
            SUGGEST.post("https://ultimatejapanese.de/db/jmdict/suggest")
            .json(&request)
            .send().unwrap()
        };
        println!("RES {:?}", res);

        Json(res.json().unwrap())
    } else {
        Json(json!({}))
    }
}


#[derive(FromForm)]
struct QueryParams {
    q: Option<String>,
    skip: Option<u32>
}

lazy_static! {
    static ref SEARCH: reqwest::Client = {
        reqwest::Client::new()
    };
    static ref SUGGEST: reqwest::Client = {
        reqwest::Client::new()
    };
}


#[get("/?<params>")]
fn search(params: QueryParams) -> Template {

    if let Some(search_term) = params.q {
        println!("Term {:?}", search_term);
        let mut res = {
            let request = build_search_request(&search_term);
            print_time!("REQUEST");
            SEARCH.post("https://ultimatejapanese.de/db/jmdict/search")
            .json(&request)
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

#[get("/")]
fn index() -> Template {
    Template::render("base", json!({}))
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, search, suggest, files])
        .attach(Template::fairing())
        .launch();
}

#[get("/<file..>")]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}
