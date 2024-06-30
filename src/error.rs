use axum::{http::StatusCode, response::IntoResponse};
use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]  
pub enum  Error {
  LoginFail,
  
  // -- auth errors
  AuthFailNoAuthTokenCookie,
  AuthFailTokenWrongFormat,
  AuthFailCtxNotInRequestExt,

  // --model errors
  TicketDeleteFailIdNotFound  { id: u64 },
}

impl  IntoResponse for Error {
  fn into_response(self) -> axum::response::Response {
    println!("->> {:<12} - {self:?}", "INTO_RES");

    (StatusCode::INTERNAL_SERVER_ERROR,"UNHANDLED_CLIENT_ERROR").into_response();
    
    //create a placeholder for the response
    let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

    //insert the Error into the response
    response.extensions_mut().insert(self);

    response 
  }
}

impl Error {
  pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
    //the line below allows for unreachable patterns such 
    // as the fallback bt in most cases we should be exhaustive
    #[allow(unreachable_patterns)]
    match self {
      // -- login fail one to one mapping
      Self::LoginFail => (StatusCode::FORBIDDEN, ClientError::LOGIN_FAIL),
      // -- Auth
      Self::AuthFailNoAuthTokenCookie | Self::AuthFailTokenWrongFormat | Self::AuthFailCtxNotInRequestExt => {
        (StatusCode::FORBIDDEN, ClientError::NO_AUTH)
      }
      // -- Model
      Self::TicketDeleteFailIdNotFound { .. } => {
        (StatusCode::BAD_REQUEST, ClientError::INVALID_PARAMS)
      }
      // -- Fallback
      _ => (
        StatusCode::INTERNAL_SERVER_ERROR,
        ClientError::SERVICE_ERROR,
      ),
    }
  }
}

#[derive(Debug, strum_macros::AsRefStr)]
#[allow(non_camel_case_types)]
pub enum ClientError {
  LOGIN_FAIL,
  NO_AUTH,
  INVALID_PARAMS,
  SERVICE_ERROR, 
}