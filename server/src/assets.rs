use axum::{http::{Uri, StatusCode, header}, response::{IntoResponse, Response}};

pub async fn page(uri: Uri) -> Response {
    let (mime, bytes): (&str, &[u8]) = match uri.path() {
        "/arena-1.webp" => ("image/webp", include_bytes!("../../client/arena-1.webp")),
        "/arena-10.webp" => ("image/webp", include_bytes!("../../client/arena-10.webp")),
        "/arena-2.webp" => ("image/webp", include_bytes!("../../client/arena-2.webp")),
        "/arena-3.webp" => ("image/webp", include_bytes!("../../client/arena-3.webp")),
        "/arena-4.webp" => ("image/webp", include_bytes!("../../client/arena-4.webp")),
        "/arena-5.webp" => ("image/webp", include_bytes!("../../client/arena-5.webp")),
        "/arena-6.webp" => ("image/webp", include_bytes!("../../client/arena-6.webp")),
        "/arena-7.webp" => ("image/webp", include_bytes!("../../client/arena-7.webp")),
        "/arena-8.webp" => ("image/webp", include_bytes!("../../client/arena-8.webp")),
        "/arena-9.webp" => ("image/webp", include_bytes!("../../client/arena-9.webp")),
        "/assets/Admin-C1_s1FpD.js" => ("text/javascript", include_bytes!("../../client/assets/Admin-C1_s1FpD.js")),
        "/assets/Match-3iMusjek.js" => ("text/javascript", include_bytes!("../../client/assets/Match-3iMusjek.js")),
        "/assets/field1-B6jM7Kva.js" => ("text/javascript", include_bytes!("../../client/assets/field1-B6jM7Kva.js")),
        "/assets/field2-D__1O5ud.js" => ("text/javascript", include_bytes!("../../client/assets/field2-D__1O5ud.js")),
        "/assets/field3-6oH1KYZR.js" => ("text/javascript", include_bytes!("../../client/assets/field3-6oH1KYZR.js")),
        "/assets/field4-uktnIwI0.js" => ("text/javascript", include_bytes!("../../client/assets/field4-uktnIwI0.js")),
        "/assets/field5-ByVQUxbT.js" => ("text/javascript", include_bytes!("../../client/assets/field5-ByVQUxbT.js")),
        "/assets/field6-Btd-Igfx.js" => ("text/javascript", include_bytes!("../../client/assets/field6-Btd-Igfx.js")),
        "/assets/field7-B3SOHZZf.js" => ("text/javascript", include_bytes!("../../client/assets/field7-B3SOHZZf.js")),
        "/assets/field8-CqtQRkzA.js" => ("text/javascript", include_bytes!("../../client/assets/field8-CqtQRkzA.js")),
        "/assets/field9-ClKWrJtp.js" => ("text/javascript", include_bytes!("../../client/assets/field9-ClKWrJtp.js")),
        "/assets/index-DeHNYzwk.js" => ("text/javascript", include_bytes!("../../client/assets/index-DeHNYzwk.js")),
        "/assets/index-ngu-J9_i.css" => ("text/css", include_bytes!("../../client/assets/index-ngu-J9_i.css")),
        "/black.png" => ("image/png", include_bytes!("../../client/black.png")),
        "/blue.png" => ("image/png", include_bytes!("../../client/blue.png")),
        "/gift-0.webp" => ("image/webp", include_bytes!("../../client/gift-0.webp")),
        "/gift-1.webp" => ("image/webp", include_bytes!("../../client/gift-1.webp")),
        "/gift-10.webp" => ("image/webp", include_bytes!("../../client/gift-10.webp")),
        "/gift-11.webp" => ("image/webp", include_bytes!("../../client/gift-11.webp")),
        "/gift-12.webp" => ("image/webp", include_bytes!("../../client/gift-12.webp")),
        "/gift-13.webp" => ("image/webp", include_bytes!("../../client/gift-13.webp")),
        "/gift-14.webp" => ("image/webp", include_bytes!("../../client/gift-14.webp")),
        "/gift-15.webp" => ("image/webp", include_bytes!("../../client/gift-15.webp")),
        "/gift-16.webp" => ("image/webp", include_bytes!("../../client/gift-16.webp")),
        "/gift-17.webp" => ("image/webp", include_bytes!("../../client/gift-17.webp")),
        "/gift-18.webp" => ("image/webp", include_bytes!("../../client/gift-18.webp")),
        "/gift-19.webp" => ("image/webp", include_bytes!("../../client/gift-19.webp")),
        "/gift-2.webp" => ("image/webp", include_bytes!("../../client/gift-2.webp")),
        "/gift-3.webp" => ("image/webp", include_bytes!("../../client/gift-3.webp")),
        "/gift-4.webp" => ("image/webp", include_bytes!("../../client/gift-4.webp")),
        "/gift-5.webp" => ("image/webp", include_bytes!("../../client/gift-5.webp")),
        "/gift-6.webp" => ("image/webp", include_bytes!("../../client/gift-6.webp")),
        "/gift-7.webp" => ("image/webp", include_bytes!("../../client/gift-7.webp")),
        "/gift-8.webp" => ("image/webp", include_bytes!("../../client/gift-8.webp")),
        "/gift-9.webp" => ("image/webp", include_bytes!("../../client/gift-9.webp")),
        "/" | "/index.html" => ("text/html; charset=utf-8", include_bytes!("../../client/index.html")),
        "/white.png" => ("image/png", include_bytes!("../../client/white.png")),
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    let cache = if uri.path().starts_with("/assets/") { "public, max-age=31536000, immutable" } else { "no-cache" };
    ([(header::CONTENT_TYPE, mime), (header::CACHE_CONTROL, cache)], bytes).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn assets_exposed_for_telegram() {
        for uri in ["/", "/white.png", "/gift-0.webp"] {
            assert_eq!(page(uri.parse().unwrap()).await.status(), StatusCode::OK);
        }
        assert_eq!(page("/server/src/main.rs".parse().unwrap()).await.status(), StatusCode::NOT_FOUND);
        assert_eq!(page("/api/missing".parse().unwrap()).await.status(), StatusCode::NOT_FOUND);
    }
}
