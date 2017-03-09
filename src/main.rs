extern crate iron;
extern crate router;
extern crate handlebars;
extern crate handlebars_iron;
extern crate markdown;
extern crate time;
extern crate staticfile;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate maplit;
extern crate mount;

extern crate rustc_serialize;

use std::io::prelude::*;
use std::fs::File;
use rustc_serialize::json::{ToJson};
use std::collections::BTreeMap;

use mount::Mount;
use iron::prelude::*;
// use iron::prelude::{Chain,Response,IronResult,Request,IronError,Iron,set_mut};
use router::Router;
use staticfile::Static;
use handlebars_iron::HandlebarsEngine;
use handlebars_iron::DirectorySource;
use handlebars_iron::Template;
use std::path::Path;

fn hello_world(req: &mut Request) -> IronResult<Response> {
    let ref article_name = req.extensions.get::<Router>()
        .unwrap().find("article").unwrap_or("");


    let mut resp = Response::new();
    let mut f = File::open(format!("./content/articles/{}.md", article_name)).unwrap();
    let mut article = String::new();
    f.read_to_string(&mut article).unwrap();

    let mut data = BTreeMap::new();
    let html = markdown::to_html(article.as_str());
    data.insert("parent".to_string(), "base".to_json());
    data.insert("title".to_string(), "Sunny Blog".to_json());
    data.insert("content".to_string(), html.to_json());

    resp.set_mut(Template::new("hellorust", data)).set_mut(iron::status::Ok);
    Ok(resp)
}

struct ResponsePrinter;

impl iron::AfterMiddleware for ResponsePrinter {
    fn catch(&self, request: &mut Request, err: IronError) -> IronResult<Response> {
        let resp_time = time::strftime("%H:%M:%S", &time::now_utc()).unwrap();
        println!("{} Error happened: {}", resp_time, err);
        println!("{} Request was: {:?}", resp_time, request);
        Err(err)
    }
}

fn main() {
    let mut router = Router::new();
    router.get("/articles/:article", hello_world, "hello");
    // router.get("/:page", get_page, "page");
    // router.post("/:page", post_page, "page");

    let mut chain = Chain::new(router);
    let printer = ResponsePrinter;
    chain.link_after(printer);

    let mut hbse = HandlebarsEngine::new();
    hbse.add(Box::new(DirectorySource::new("./content/templates/", ".hbs")));

    // load templates from all registered sources
    if let Err(r) = hbse.reload() {
        panic!("{}", r);
    }

    chain.link_after(hbse);

    let mut mount = Mount::new();
    mount
        .mount("/", chain)
        .mount("/public", Static::new(Path::new("./public")));
    println!("Listening on 3000");
    Iron::new(mount).http("localhost:3000").unwrap();
}
