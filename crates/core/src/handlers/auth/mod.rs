use actix_web::{
    cookie::{time::OffsetDateTime, Cookie, Expiration},
    get, post, web, HttpRequest, HttpResponse, Responder,
};
use chrono::Utc;
use sea_query::{Expr, Query};
use serde::Deserialize;
use serde_json::json;

use crate::{
    auth::{self, TokenType},
    config,
    entities::{Queries, RefreshTokenTree, User, UserIden},
    handlers::Error,
    id::Id,
};

pub mod oauth2;
pub mod token;

#[derive(Deserialize)]
pub struct SignupBody {
    #[serde(rename = "firstName")]
    first_name: String,
    #[serde(rename = "lastName")]
    last_name: String,
    username: String,
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct LoginBody {
    email: String,
    password: String,
}

#[post("/auth/signup")]
pub async fn signup(
    body: web::Json<SignupBody>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> impl Responder {
    let user = User {
        id: Id::new(),
        first_name: body.first_name.clone(),
        last_name: body.last_name.clone(),
        username: body.username.clone(),
        email: body.email.clone(),
        password: body.password.clone(),
        verified: false,
        banned: false,
        created_at: Utc::now(),
        updated_at: None,
    };

    let query = user.query_insert();
    if let Err(error) = query {
        match error {
            Some(errors) => {
                return HttpResponse::BadRequest().json(json!({
                    "message": "Validation failed",
                    "errors": errors
                }));
            }
            None => {
                return HttpResponse::InternalServerError().json(Error {
                    message: "An error occurred while hashing the password".to_string(),
                });
            }
        }
    }

    let Ok(_) = db_pool.get().await.unwrap().execute(query.unwrap().as_str(), &[]).await else {
        return HttpResponse::InternalServerError().json(Error {
            message: "An error occurred while inserting the user".to_string(),
        });
    };

    HttpResponse::Ok().json(json!({
        "success": true,
        "user": user
    }))
}

#[post("/auth/login")]
pub async fn login(
    body: web::Json<LoginBody>,
    config: web::Data<config::Config>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> impl Responder {
    let Ok(rows) = db_pool.get().await.unwrap().query(User::query_select(Query::select().and_where(Expr::col(UserIden::Email).like(body.email.clone())).limit(1)).as_str(), &[]).await else {
        return HttpResponse::InternalServerError().json(Error {
            message: "An error occurred while fetching the user".to_string(),
        });
    };

    let Some(row) = rows.iter().next() else {
        return HttpResponse::BadRequest().json(Error {
            message: "No user was found".to_string(),
        });
    };

    let user = User::from(row);

    let Ok(is_valid) = auth::verify_password(body.password.as_str(), &user.password) else {
        return HttpResponse::BadRequest().json(Error {
            message: "Unable to parse password hash".to_string(),
        });
    };
    if !is_valid {
        return HttpResponse::BadRequest().json(Error {
            message: "No user was found with this email/password combo".to_string(),
        });
    }

    let Ok(access_token) = TokenType::Access.sign(config.clone(), user.clone()) else {
        return HttpResponse::InternalServerError().json(Error {
            message: "An error occurred while signing the access token".to_string(),
        });
    };

    let Ok(refresh_token) = TokenType::Refresh.sign(config, user.clone()) else {
        return HttpResponse::InternalServerError().json(Error {
            message: "An error occurred while signing the refresh token".to_string(),
        });
    };

    let refresh_token_tree = RefreshTokenTree {
        id: Id::new().to_string(),
        user_id: user.id.clone(),
        inactive_at: Utc::now() + chrono::Duration::days(15),
        expires_at: Utc::now() + chrono::Duration::days(90),
        tokens: vec![refresh_token.clone().claims.jti],
        created_at: Utc::now(),
        updated_at: None,
    };

    let query = refresh_token_tree.query_insert();
    let Ok(_) = db_pool.get().await.unwrap().execute(query.unwrap().as_str(), &[]).await else {
        return HttpResponse::InternalServerError().json(Error {
            message: "An error occurred while inserting the refresh token tree".to_string(),
        });
    };

    let Ok(cookie_expiration) = OffsetDateTime::from_unix_timestamp(refresh_token.expires_at.timestamp()) else {
        return HttpResponse::InternalServerError().json(Error {
            message: "An error occurred while parsing the cookie expiration".to_string(),
        });
    };

    HttpResponse::Ok()
        .cookie(
            Cookie::build("refreshToken", refresh_token.token)
                .secure(true)
                .http_only(true)
                .expires(Expiration::from(cookie_expiration))
                .finish(),
        )
        .json(json!({
            "success": true,
            "user": user,
            "accessToken": access_token.clone().token,
        }))
}

#[get("/auth/logout")]
pub async fn logout(req: HttpRequest) -> impl Responder {
    let Ok(cookies) = req.cookies() else {
        return HttpResponse::InternalServerError().json(Error {
            message: "An error occurred while fetching the cookies".to_string(),
        });
    };

    let Some(cookie) = cookies.iter().find(|cookie| cookie.name() == "refreshToken") else {
        return HttpResponse::BadRequest().json(Error {
            message: "No refresh token was found".to_string(),
        });
    };

    let mut cookie = cookie.clone();
    cookie.make_removal();

    HttpResponse::Ok().cookie(cookie).json(json!({
        "success": true
    }))
}
