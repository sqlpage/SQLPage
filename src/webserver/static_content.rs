use crate::utils::static_filename;
use actix_web::{
    http::header::{
        CacheControl, CacheDirective, ContentEncoding, ETag, EntityTag, Header, IfNoneMatch,
    },
    web, HttpRequest, HttpResponse, Resource,
};

macro_rules! static_file_endpoint {
    ($filestem:literal, $extension:literal, $mime:literal) => {{
        const FILENAME_WITH_TAG: &str = static_filename!(concat!($filestem, ".", $extension));
        web::resource(FILENAME_WITH_TAG).to(|req: HttpRequest| async move {
            let file_etag = EntityTag::new_strong(FILENAME_WITH_TAG.to_string());
            if matches!(IfNoneMatch::parse(&req), Ok(IfNoneMatch::Items(etags)) if etags.iter().any(|etag| etag.weak_eq(&file_etag))) {
                return HttpResponse::NotModified().finish();
            }
            HttpResponse::Ok()
                .content_type(concat!($mime, ";charset=UTF-8"))
                .insert_header(CacheControl(vec![
                    CacheDirective::Public,
                    CacheDirective::MaxAge(3600 * 24 * 7),
                    CacheDirective::Extension("immutable".to_owned(), None),
                ]))
                .insert_header(ETag(file_etag))
                .insert_header(ContentEncoding::Gzip)
                .body(
                    &include_bytes!(concat!(env!("OUT_DIR"), "/", $filestem, ".", $extension))[..],
                )
        })
    }};
}

pub fn js() -> Resource {
    static_file_endpoint!("sqlpage", "js", "application/javascript")
}

pub fn apexcharts_js() -> Resource {
    static_file_endpoint!("apexcharts", "js", "application/javascript")
}

pub fn css() -> Resource {
    static_file_endpoint!("sqlpage", "css", "text/css")
}

pub fn icons() -> Resource {
    static_file_endpoint!("tabler-icons", "svg", "image/svg+xml")
}
