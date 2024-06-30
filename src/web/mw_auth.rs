use axum::async_trait;
use axum::body::Body;
use axum::extract::{FromRequestParts, Request, State};
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::Response;
use lazy_regex::regex_captures;
use tower_cookies::{Cookie, Cookies};

use crate::ctx::Ctx;
use crate::model::ModelController;
use crate::web::AUTH_TOKEN;
use crate::{Error, Result};

pub async fn mw_require_auth(
  ctx: Result<Ctx>, 
  req: Request<Body>, 
  next: Next,
) -> Result<Response> {
  println!("->> {:<12} - mw_require_auth", "MIDDLEWARE");
  //getting the cookie and returning it as a string
 ctx?;

  Ok(next.run(req).await)
} 

pub async fn mw_ctx_resolver(
  _mc: State<ModelController>,
  cookies: Cookies,
  mut req: Request<Body>,
  next: Next,
) -> Result<Response>{
  println!("->> {:<12} --mw_ctx_resolver", "MIDDLEWARE");

  let auth_token = cookies.get(AUTH_TOKEN)
    .map(|c| c.value().to_string());

  // compute Result<Ctx>
  let result_ctx = match auth_token
    .ok_or(Error::AuthFailNoAuthTokenCookie)
    .and_then(|token| parse_token(token))
   {
    Ok((user_id, _exp, _sign)) => {
      //TODO: Token components validation
      Ok(Ctx::new(user_id))
    }
    Err(e) => Err(e),
  };

  // let result_ctx = if let Some(auth_token) = auth_token {
  //   match parse_token(auth_token) {
  //         Ok((user_id, _exp, _sign)) => {
  //             // TODO: Token components validation
  //             Ok(Ctx::new(user_id))
  //         },
  //         Err(e) => Err(e),
  //     }
  // } else {
  //     Err(Error::AuthFailNoAuthTokenCookie)
  // };

  //remove the cookie if something went wrong other than NoAuthTokenCookie
  if result_ctx.is_err()
    && !matches!(result_ctx, Err    (Error::AuthFailNoAuthTokenCookie)) 
  {
    cookies.remove(Cookie::from(AUTH_TOKEN));
  }
  // Store the cookie result in the request extension
  req.extensions_mut().insert(result_ctx);

  Ok(next.run(req).await)
}

// region: --- Ctx Extractor
#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Ctx {
  type Rejection = Error;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self>{ 
    println!("->> {:<12} -- Ctx", "EXTRACTOR");

    parts 
      .extensions
      .get::<Result<Ctx>>()
      .ok_or(Error::AuthFailCtxNotInRequestExt)?
      .clone()
  }
}
// endregion --- Ctx Extracrtor

/// Parse a token of format `user-[user-id].[expiration].[signature]`
/// Returns {user_id, expiration, signature}

fn parse_token(token: String) -> Result<(u64, String, String)>{
  if let Some((_whole, user_id, exp, sign)) = regex_captures!(
    r#"^user-(\d+)\.(.+)\.(.+)$"#, // literal regex
    &token
  ) {
    let user_id: u64 = user_id.parse().map_err(|_| Error::AuthFailTokenWrongFormat)?;
    Ok((user_id, exp.to_string(), sign.to_string()))
  } else {
    Err(Error::AuthFailTokenWrongFormat)
  }
}