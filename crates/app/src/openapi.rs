use adrastos_core::entities;
use utoipa::OpenApi;

use crate::handlers;

#[derive(OpenApi)]
#[openapi(
    paths(handlers::tables::create),
    components(schemas(
        entities::custom_table::fields::Field,
        entities::custom_table::fields::FieldInfo,
        entities::custom_table::fields::RelationTarget,
        handlers::tables::CreateBody
    ))
)]
pub struct ApiDoc;
