use actix_web::{dev::ServiceRequest, HttpMessage};
use actix_web_httpauth::extractors::{
    bearer::{BearerAuth, Config as BearerConfig},
    AuthenticationError,
};
use jsonwebtoken::{
    errors::ErrorKind,
    jwk::{AlgorithmParameters, JwkSet},
    DecodingKey, TokenData, Validation,
};
use lazy_static::lazy_static;
use serde::Deserialize;
use std::fmt::Display;

lazy_static! {
    static ref VALIDATION: Validation = {
        let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.set_audience(&["account"]);

        validation
    };
}

#[derive(Debug, Deserialize, Clone)]
pub struct Claims {
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
    pub aud: String,
    pub sub: String,
}

#[derive(Debug)]
enum JwtError {
    InvalidToken,
    ExpiredToken,
    KeyLoadError(KeyLoadError),
}

#[derive(Debug)]
enum KeyLoadError {
    Fetch(reqwest::Error),
    NotFound,
    BadParameters,
}

impl Display for JwtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JwtError::InvalidToken => write!(f, "Token is invalid"),
            JwtError::ExpiredToken => write!(f, "Token is expired"),
            JwtError::KeyLoadError(kind) => write!(f, "Failed to load key: {kind}"),
        }
    }
}

impl Display for KeyLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyLoadError::Fetch(inner) => inner.fmt(f),
            KeyLoadError::BadParameters => write!(f, "Bad parameters"),
            KeyLoadError::NotFound => write!(f, "Key not found"),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for JwtError {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        match value.kind() {
            ErrorKind::ExpiredSignature => JwtError::ExpiredToken,
            _ => JwtError::InvalidToken,
        }
    }
}

impl From<KeyLoadError> for JwtError {
    fn from(value: KeyLoadError) -> Self {
        JwtError::KeyLoadError(value)
    }
}

pub async fn validate(
    request: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    let token = credentials.token();
    if let Ok(decoded) = decode_token(token).await {
        request.extensions_mut().insert(decoded);
        Ok(request)
    } else {
        let config = request
            .app_data::<BearerConfig>()
            .cloned()
            .unwrap_or_default();

        Err((AuthenticationError::from(config).into(), request))
    }
}

async fn decode_token(token: &str) -> Result<TokenData<Claims>, JwtError> {
    let key = load_key("5oTWTw5vJJz1QZuKgMNfZLr96jgMvmt0quwDAxMQGJQ").await?;

    jsonwebtoken::decode::<Claims>(token, &key, &VALIDATION).map_err(|e| e.into())
}

async fn load_key(kid: &str) -> Result<DecodingKey, KeyLoadError> {
    let jwks_url = "http://localhost:8081/realms/Test/protocol/openid-connect/certs";
    let jwks = reqwest::get(jwks_url)
        .await
        .map_err(|e| KeyLoadError::Fetch(e))?
        .json::<JwkSet>()
        .await
        .map_err(|e| KeyLoadError::Fetch(e))?;

    let jwk: &jsonwebtoken::jwk::Jwk = jwks.find(kid).ok_or(KeyLoadError::NotFound)?;

    if let AlgorithmParameters::RSA(rsa_params) = &jwk.algorithm {
        DecodingKey::from_rsa_components(&rsa_params.n, &rsa_params.e)
            .map_err(|_| KeyLoadError::BadParameters)
    } else {
        todo!()
    }
}
