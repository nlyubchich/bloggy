extern crate iron;
extern crate router;
extern crate handlebars;
extern crate handlebars_iron;
extern crate markdown;
extern crate time;
extern crate staticfile;
#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate mount;

use router::NoRoute;
use std::io::prelude::*;
use std::fs::File;
use mount::Mount;
use iron::prelude::*;
use router::Router;
use staticfile::Static;
use handlebars_iron::HandlebarsEngine;
use handlebars_iron::DirectorySource;
use handlebars_iron::Template;
use std::path::Path;
use std::fs;

#[derive(Deserialize)]
struct Config {
    title: String,
    tags: std::vec::Vec<String>,
}


#[derive(Serialize, Debug)]
struct BaseTemplateScheme {
    parent: String,
}

#[derive(Serialize, Debug)]
struct Article {
    content: String,
    title: String,
    parent: String,
    endpoint: String,
    tags: std::vec::Vec<String>,
}

#[derive(Serialize, Debug)]
struct ArticleList {
    articles: std::vec::Vec<Article>,
    parent: String,
}

fn article_handler(req: &mut Request) -> IronResult<Response> {
    let ref article_name = req.extensions.get::<Router>()
        .unwrap().find("article").unwrap_or("");

    let article_path = format!("./content/articles/{}", article_name);
    let mut resp = Response::new();
    let mut f = File::open(format!("{}/full.md", article_path)).unwrap();
    let mut article = String::new();
    f.read_to_string(&mut article).unwrap();


    let mut article_meta_file = File::open(format!("{}/meta.toml", article_path)).unwrap();
    let mut article_toml_str = String::new();
    article_meta_file.read_to_string(&mut article_toml_str).unwrap();
    let meta: Config = toml::from_str(article_toml_str.as_str()).unwrap();

    let html = markdown::to_html(article.as_str());
    resp.set_mut(Template::new("article", Article {
        parent: "base".to_string(),
        endpoint: article_name.to_string(),
        title: meta.title,
        content: html,
        tags: meta.tags.to_vec(),
    })).set_mut(iron::status::Ok);
    Ok(resp)
}


fn article_list_handler(_: &mut Request) -> IronResult<Response> {
    let mut articles = vec![];
    let mut resp = Response::new();
    let paths = fs::read_dir("./content/articles").unwrap();

    for path in paths {
        let article_path = path.unwrap().path();
        let mut f = File::open(format!("{}/preview.md", article_path.to_str().unwrap())).unwrap();
        let mut article = String::new();
        f.read_to_string(&mut article).unwrap();


        let mut article_meta_file = File::open(format!("{}/meta.toml", article_path.to_str().unwrap())).unwrap();
        let mut article_toml_str = String::new();
        article_meta_file.read_to_string(&mut article_toml_str).unwrap();
        let meta: Config = toml::from_str(article_toml_str.as_str()).unwrap();

        let html = markdown::to_html(article.as_str());
        articles.push(Article {
            endpoint: article_path.to_str().unwrap().to_string().split('/').last().unwrap().to_string(),
            title: meta.title,
            content: html,
            parent: "base".to_string(),
            tags: meta.tags,
        });
    }
    resp.set_mut(Template::new("list", ArticleList {
        articles: articles,
        parent: "base".to_string(),
    })).set_mut(iron::status::Ok);
    Ok(resp)
}

struct ResponsePrinter;

impl iron::AfterMiddleware for ResponsePrinter {
    fn catch(&self, request: &mut Request, err: IronError) -> IronResult<Response> {
        let resp_time = time::strftime("%H:%M:%S", &time::now_utc()).unwrap();
        println!("{} Error happened: {}\n{:?}", resp_time, err, request);
        Err(err)
    }
}

struct Custom404;

impl iron::AfterMiddleware for Custom404 {
    fn catch(&self, _: &mut Request, err: IronError) -> IronResult<Response> {
        if let Some(_) = err.error.downcast::<NoRoute>() {
            Ok(Response::with((
                iron::status::NotFound,
                Template::new("404", BaseTemplateScheme {
                    parent: "base".to_string(),
                })
            )))
        } else {
            Err(err)
        }
    }
}

fn main() {
    let mut router = Router::new();
    router.get("/article/:article", article_handler, "article");
    router.get("/", article_list_handler, "article_list");

    let mut chain = Chain::new(router);
    let printer = ResponsePrinter;
    chain.link_after(printer);
    chain.link_after(Custom404);

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
