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
        entities::custom_table::fields::StringField,
        entities::custom_table::fields::NumberField,
        entities::custom_table::fields::BooleanField,
        entities::custom_table::fields::DateField,
        entities::custom_table::fields::EmailField,
        entities::custom_table::fields::UrlField,
        entities::custom_table::fields::SelectField,
        entities::custom_table::fields::RelationField,
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
