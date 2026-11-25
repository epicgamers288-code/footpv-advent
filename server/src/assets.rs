use axum::{http::{Uri, StatusCode, header}, response::{IntoResponse, Response}};

pub async fn page(uri: Uri) -> Response {
    let (mime, bytes): (&str, &[u8]) = match uri.path() {
        "/assets/Admin-DtswxAwE.js" => ("text/javascript", include_bytes!("../../client/assets/Admin-DtswxAwE.js")),
        "/assets/Match-gtG-CE-J.js" => ("text/javascript", include_bytes!("../../client/assets/Match-gtG-CE-J.js")),
        "/assets/field1-MrAjHzyP.js" => ("text/javascript", include_bytes!("../../client/assets/field1-MrAjHzyP.js")),
        "/assets/field2-B2A4du1A.js" => ("text/javascript", include_bytes!("../../client/assets/field2-B2A4du1A.js")),
        "/assets/field3-YUEcuaKT.js" => ("text/javascript", include_bytes!("../../client/assets/field3-YUEcuaKT.js")),
        "/assets/field4-CaXJMLlm.js" => ("text/javascript", include_bytes!("../../client/assets/field4-CaXJMLlm.js")),
        "/assets/field5-BQIR_6ti.js" => ("text/javascript", include_bytes!("../../client/assets/field5-BQIR_6ti.js")),
        "/assets/field6-Bk60vh0V.js" => ("text/javascript", include_bytes!("../../client/assets/field6-Bk60vh0V.js")),
        "/assets/field7-D2zUv0EP.js" => ("text/javascript", include_bytes!("../../client/assets/field7-D2zUv0EP.js")),
        "/assets/field8-B4pDXifB.js" => ("text/javascript", include_bytes!("../../client/assets/field8-B4pDXifB.js")),
        "/assets/field9-Cgqg6JbQ.js" => ("text/javascript", include_bytes!("../../client/assets/field9-Cgqg6JbQ.js")),
        "/assets/index-DeQQRPhj.css" => ("text/css", include_bytes!("../../client/assets/index-DeQQRPhj.css")),
        "/assets/index-YnHp7e1X.js" => ("text/javascript", include_bytes!("../../client/assets/index-YnHp7e1X.js")),
        "/black.png" => ("image/png", include_bytes!("../../client/black.png")),
        "/blue.png" => ("image/png", include_bytes!("../../client/blue.png")),
        "/gift-0.webp" => ("image/webp", include_bytes!("../../client/gift-0.webp")),
        "/gift-1.webp" => ("image/webp", include_bytes!("../../client/gift-1.webp")),
        "/gift-2.webp" => ("image/webp", include_bytes!("../../client/gift-2.webp")),
        "/gift-3.webp" => ("image/webp", include_bytes!("../../client/gift-3.webp")),
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
