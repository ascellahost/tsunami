use crate::{
    database::{
        queries::{get_user_auth, get_user_token},
        structs::Users,
    },
    prelude::CLIENT,
};
use actix_web::{
    body::BoxBody, dev::Payload, http::header::HeaderMap, FromRequest, HttpRequest, HttpResponse, Responder,
    ResponseError,
};
use anyhow::Result;
use futures::Future;
use paperclip::{
    actix::{web, Apiv2Schema, Apiv2Security, OperationModifier},
    v2::{
        models::{DefaultOperationRaw, DefaultSchemaRaw, Either},
        schema::Apiv2Errors,
    },
};
use rand::{distributions::Alphanumeric, Rng};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    fmt::{self, Display},
    pin::Pin,
};
use validator::{ValidationErrors, ValidationErrorsKind};

//Users

#[derive(Apiv2Security)]
#[openapi(apiKey, in = "header", name = "Authorization", description = "Use format 'TOKEN'")]
pub struct AccessToken {
    inner: Users,
}
impl AccessToken {
    pub fn inner(self) -> Users {
        self.inner
    }
    pub fn discord_id(&self) -> String {
        self.inner.discord_id.clone()
    }
    pub fn domain(&self) -> String {
        self.inner.domain.clone()
    }
    pub fn id(&self) -> i32 {
        self.inner.id
    }
    pub fn key(&self) -> String {
        self.inner.key.clone()
    }
    pub fn name(&self) -> String {
        self.inner.name.clone()
    }
    pub fn autodelete(&self) -> Option<i32> {
        self.inner.autodelete
    }
    pub fn deleteall(&self) -> Option<i32> {
        self.inner.deleteall
    }
    pub fn upload_key(&self) -> Option<String> {
        self.inner.upload_key.clone()
    }
    pub fn url_style(&self) -> i32 {
        self.inner.url_style
    }
}

trait GetHeaderFn {
    fn get_hdr(&self, key: &str) -> Option<String>;
}

impl GetHeaderFn for HeaderMap {
    fn get_hdr(&self, key: &str) -> Option<String> {
        self.get(key)?.to_str().map(|x| x.to_owned()).ok()
    }
}

#[derive(Deserialize, Clone)]
pub struct Auth {
    pub auth: String,
}

impl FromRequest for AccessToken {
    type Error = Error;

    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let path = req.query_string().to_owned();
        let headers = req.headers();

        let auth = headers.get_hdr("Authorization").unwrap_or_default();
        let user_id = headers
            .get_hdr("x-user-id")
            .unwrap_or_else(|| "-1".to_string())
            .parse::<i32>()
            .unwrap();
        let user_token = headers.get_hdr("x-user-token").unwrap_or_default();

        Box::pin(async move {
            if !auth.is_empty() {
                if let Ok(user) = get_user_auth::exec(auth.to_owned()).await {
                    return Ok(Self { inner: user });
                }
            }

            if user_id != -1 || !user_token.is_empty() {
                if let Ok(user) = get_user_token::exec(user_id, user_token.to_owned()).await {
                    return Ok(Self { inner: user });
                }
            }
            if let Ok(res) = web::Query::<Auth>::from_query(&path) {
                if let Ok(user) = get_user_auth::exec(res.auth.to_owned()).await {
                    return Ok(Self { inner: user });
                }
            }

            Err(Error::NotAuthorized)
        })
    }

    fn extract(req: &HttpRequest) -> Self::Future {
        Self::from_request(req, &mut Payload::None)
    }
}
#[derive(Serialize, Debug)]
#[serde(tag = "error")]
pub enum Error {
    FileTooLarge { max_size: usize },
    FileTypeNotAllowed,
    FailedToReceive,
    ValidationError(HashMap<&'static str, ValidationErrorsKind>),
    NotAuthorized,
    BlockingError,
    DatabaseError,
    MissingData,
    UnknownTag,
    BadAnyhowRequest { message: String },
    BadRequest,
    ProbeError,
    RateLimit { message: String },
    NotFound,
    IOError,
    LabelMe,
    Four04 { message: String },
}

impl From<ValidationErrors> for Error {
    fn from(err: ValidationErrors) -> Self {
        Error::ValidationError(err.errors().to_owned())
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::BadAnyhowRequest {
            message: err.to_string(),
        }
    }
}

impl Apiv2Errors for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, failed)", self.status_code())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl Error {
    fn message(&self) -> String {
        match &self {
            Error::FileTooLarge { max_size } => format!("File too large {max_size}"),
            Error::FileTypeNotAllowed => "Bad request".into(),
            Error::NotAuthorized => "Not authorized".into(),
            Error::FailedToReceive => "Failed to receive".into(),
            Error::DatabaseError => "Database Error".into(),
            Error::MissingData => "Missing data".into(),
            Error::ProbeError => "Probe error".into(),
            Error::NotFound => "Not found".into(),
            Error::BlockingError => "Internal server error".into(),
            Error::BadRequest => "Bad request".into(),
            Error::RateLimit { message } => format!("Rate limit exceeded {message}"),
            Error::Four04 { message } => format!("404: {message}"),
            Error::BadAnyhowRequest { message } => message.to_string(),
            Error::ValidationError(..) => "Validation error".into(),
            _ => "".into(),
        }
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match &self {
            Error::FileTooLarge { .. } => StatusCode::PAYLOAD_TOO_LARGE,
            Error::FileTypeNotAllowed => StatusCode::BAD_REQUEST,
            Error::NotAuthorized => StatusCode::FORBIDDEN,
            Error::FailedToReceive => StatusCode::BAD_REQUEST,
            Error::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            Error::MissingData => StatusCode::BAD_REQUEST,
            Error::UnknownTag => StatusCode::BAD_REQUEST,
            Error::ProbeError => StatusCode::INTERNAL_SERVER_ERROR,
            Error::NotFound => StatusCode::NOT_FOUND,
            Error::BlockingError => StatusCode::INTERNAL_SERVER_ERROR,
            Error::IOError => StatusCode::INTERNAL_SERVER_ERROR,
            Error::LabelMe => StatusCode::INTERNAL_SERVER_ERROR,
            Error::BadRequest => StatusCode::BAD_REQUEST,
            Error::BadAnyhowRequest { .. } => StatusCode::BAD_REQUEST,
            Error::ValidationError(..) => StatusCode::BAD_REQUEST,
            Error::RateLimit { .. } => StatusCode::TOO_MANY_REQUESTS,
            Error::Four04 { .. } => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse {
        // let body = serde_json::to_string(&self).unwrap();
        let body = serde_json::to_string(&json!({
            "code": &self.status_code().as_u16(),
            "error": &self.message(),
            "success": false
        }))
        .unwrap();
        HttpResponse::build(self.status_code())
            .content_type("application/json")
            .body(body)
    }
}

pub fn create_config<T: std::fmt::Display>(auth: T) -> serde_json::Value {
    serde_json::json!({
      "Version": "13.1.0",
      "Name": "Ascella.host - images",
      "DestinationType": "ImageUploader",
      "RequestMethod": "POST",

      "RequestURL": "https://ascella.wtf/v2/ascella/upload",
      "Body": "MultipartFormData",
      "Headers": {
        "authorization": auth.to_string(),
      },
      "FileFormName": "image",
      "URL": "$json:url$",
    })
}
#[derive(Serialize, Deserialize, Apiv2Schema)]
pub struct SendMessage {
    #[openapi(example = "200")]
    code: i32,
    success: bool,
    #[openapi(example = "Succesful Response")]
    message: &'static str,
}
impl SendMessage {
    pub fn new(code: i32, success: bool, message: &'static str) -> Self {
        SendMessage { code, success, message }
    }
}

#[derive(Serialize, Deserialize, Apiv2Schema)]
pub struct UploadSuccess {
    #[openapi(example = "200")]
    code: i32,
    #[openapi(example = "true")]
    success: bool,
    #[openapi(example = "https://ascella.host/tdaCZHo1")]
    url: String,
    #[openapi(example = "https://ascella.wtf/v2/ascella/view/tdaCZHo1.png")]
    raw: String,
}
impl UploadSuccess {
    pub fn new(vanity: &str, domain: &str) -> Self {
        Self {
            code: 200,
            success: true,
            url: format!("{}/{}", domain, vanity),
            raw: format!("https://ascella.wtf/v2/ascella/view/{}.png", vanity),
        }
    }
}

pub fn ran_str(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

pub async fn send_text_webhook<T: Display>(data: T) -> Result<()> {
    let json = serde_json::json!({ "content": data.to_string() });
    log_shit(json).await?;
    Ok(())
}

pub async fn log_shit(data: Value) -> Result<()> {
    // Set up and send the post request
    CLIENT
        .post(dotenv::var("WEBHOOK").unwrap())
        .header("Content-Type", "application/json")
        .header("User-Agent", "Ascella.host/v2 Ascella is a fast image uploader")
        .body(data.to_string())
        .send()
        .await?;

    Ok(())
}
pub struct OkResponse<T: Serialize + Apiv2Schema>(pub T);

impl<T> fmt::Debug for OkResponse<T>
where
    T: fmt::Debug + Serialize + Apiv2Schema + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status: StatusCode = StatusCode::CREATED;
        let status_str = status.canonical_reason().unwrap_or(status.as_str());
        f.write_fmt(std::fmt::Arguments::new_v1(
            &[],
            &[
                std::fmt::ArgumentV1::new(&(status_str), std::fmt::Display::fmt),
                std::fmt::ArgumentV1::new(&(self.0), std::fmt::Display::fmt),
            ],
        ))
    }
}
impl<T> fmt::Display for OkResponse<T>
where
    T: fmt::Display + Serialize + Apiv2Schema,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}
impl<T> Responder for OkResponse<T>
where
    T: Serialize + Apiv2Schema,
{
    type Body = BoxBody;
    fn respond_to(self, _: &HttpRequest) -> HttpResponse<BoxBody> {
        let status: StatusCode = StatusCode::OK;
        let body = match serde_json::to_string(&self.0) {
            Ok(body) => body,
            Err(e) => return e.error_response(),
        };
        HttpResponse::build(status).content_type("application/json").body(body)
    }
}

impl<T> Apiv2Schema for OkResponse<T>
where
    T: Serialize + Apiv2Schema,
{
    fn name() -> Option<String> {
        T::name()
    }
    fn raw_schema() -> DefaultSchemaRaw {
        T::raw_schema()
    }
}
use paperclip::v2::{models::Response, schema::Apiv2Schema};

impl<T> OperationModifier for OkResponse<T>
where
    T: Serialize + Apiv2Schema,
{
    fn update_response(op: &mut DefaultOperationRaw) {
        let status: StatusCode = StatusCode::CREATED;
        op.responses.insert(
            status.as_str().into(),
            Either::Right(Response {
                description: status.canonical_reason().map(ToString::to_string),
                schema: Some({
                    let mut def = T::schema_with_ref();
                    def.retain_ref();
                    def
                }),
                ..Default::default()
            }),
        );
    }
}
