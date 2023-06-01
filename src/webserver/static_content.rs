use crate::utils::static_filename;
use actix_web::{
    http::header::{CacheControl, CacheDirective},
    web, HttpResponse, Resource,
};

pub fn js() -> Resource {
    web::resource(static_filename!("sqlpage.js")).to(|| async {
        HttpResponse::Ok()
            .content_type("text/javascript;charset=UTF-8")
            .insert_header(CacheControl(vec![
                CacheDirective::Public,
                CacheDirective::MaxAge(7 * 24 * 3600),
                CacheDirective::Extension("immutable".to_owned(), None),
            ]))
            .body(&include_bytes!(concat!(env!("OUT_DIR"), "/sqlpage.js"))[..])
    })
}

pub fn css() -> Resource {
    web::resource(static_filename!("sqlpage.css")).to(|| async {
        HttpResponse::Ok()
            .content_type("text/css;charset=UTF-8")
            .insert_header(CacheControl(vec![
                CacheDirective::Public,
                CacheDirective::MaxAge(7 * 24 * 3600),
                CacheDirective::Extension("immutable".to_owned(), None),
            ]))
            .body(&include_bytes!(concat!(env!("OUT_DIR"), "/sqlpage.css"))[..])
    })
}
