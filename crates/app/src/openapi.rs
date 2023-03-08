use core::entities;
use utoipa::{
    openapi::{Object, ObjectBuilder, SchemaFormat, SchemaType},
    OpenApi,
};

use crate::handlers;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::auth::signup,
        handlers::auth::login,
        handlers::auth::token::refresh,
        handlers::tables::create,
        handlers::tables::delete,
    ),
    components(schemas(
        entities::User,
        entities::Connection,
        entities::RefreshTokenTree,
        entities::custom_table::schema::CustomTableSchema,
        entities::custom_table::schema::StringField,
        entities::custom_table::schema::NumberField,
        entities::custom_table::schema::BooleanField,
        entities::custom_table::schema::DateField,
        entities::custom_table::schema::EmailField,
        entities::custom_table::schema::UrlField,
        entities::custom_table::schema::SelectField,
        entities::custom_table::schema::RelationField,
        // handlers::Error,
        handlers::auth::SignupBody,
        handlers::auth::LoginBody,
        handlers::tables::CreateBody,
    ))
)]
pub struct ApiDoc;

pub fn email() -> Object {
    ObjectBuilder::new()
        .schema_type(SchemaType::String)
        .format(Some(SchemaFormat::Custom("email".into())))
        .build()
}
