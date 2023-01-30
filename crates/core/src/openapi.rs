use utoipa::{
    openapi::{Object, ObjectBuilder, SchemaFormat, SchemaType},
    OpenApi,
};

use crate::{entities, handlers};

#[derive(OpenApi)]
#[openapi(
    paths(handlers::auth::signup, handlers::auth::login),
    components(schemas(entities::User, handlers::auth::SignupBody, handlers::auth::LoginBody))
)]
pub struct ApiDoc;

pub fn email() -> Object {
    ObjectBuilder::new()
        .schema_type(SchemaType::String)
        .format(Some(SchemaFormat::Custom("email".into())))
        .build()
}
