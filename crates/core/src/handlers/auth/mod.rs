use actix_web::{post, web, HttpResponse, Responder};
use chrono::Utc;
use sea_query::{Expr, Query};
use serde::Deserialize;

use crate::{
    auth,
    entities::{Queries, User, UserIden},
    handlers::Error,
    id::Id,
};

pub mod oauth2;

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

    let Ok(query) = user.query_insert() else {
        return HttpResponse::BadRequest().json(Error {
            message: "Bad request".to_string(), // TODO(@Xenfo): return a more specific error
        });
    };

    let Ok(_) = db_pool.get().await.unwrap().execute(query.as_str(), &[]).await else {
        return HttpResponse::InternalServerError().json(Error {
            message: "An error occurred while inserting the user".to_string(),
        });
    };

    HttpResponse::Ok().json(user)
}

#[post("/auth/login")]
pub async fn login(
    body: web::Json<LoginBody>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> impl Responder {
    let Ok(rows) = db_pool.get().await.unwrap().query(User::query_select(Query::select().columns(vec![
        UserIden::Id,
        UserIden::FirstName,
        UserIden::LastName,
        UserIden::Email,
        UserIden::Username,
        UserIden::Password,
        UserIden::Verified,
        UserIden::Banned,
        UserIden::CreatedAt,
        UserIden::UpdatedAt,
    ]).and_where(Expr::col(UserIden::Email).like(body.email.clone())).limit(1)).as_str(), &[]).await else {
        return HttpResponse::InternalServerError().json(Error {
            message: "An error occurred while inserting the user".to_string(),
        });
    };

    let Some(row) = rows.iter().next() else {
        return HttpResponse::BadRequest().json(Error {
            message: "Bad request".to_string(), // TODO(@Xenfo): return a more specific error
        });
    };

    let user = User::from(row);

    let Ok(is_valid) = auth::verify_password(body.password.as_str(), &user.password) else {
        return HttpResponse::BadRequest().json(Error {
            message: "Bad request".to_string(), // TODO(@Xenfo): return a more specific error
        });
    };

    if !is_valid {
        return HttpResponse::BadRequest().json(Error {
            message: "Bad request".to_string(), // TODO(@Xenfo): return a more specific error
        });
    }

    HttpResponse::Ok().json(user)
}
