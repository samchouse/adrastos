use json_patch::{AddOperation, PatchOperation};
use jsonptr::Pointer;
use serde_json::json;

pub use adrastos_core::util::*;

pub struct PaginationInfo {
    pub page: u64,
    pub limit: u64,
    pub count: u64,
}

pub fn attach_pagination_details(target: &mut serde_json::Value, info: PaginationInfo) {
    let mut diff = json_patch::diff(
        target,
        &json!({
            "pagination": {
                "records": info.count,
                "pages": (info.count as f64 / info.limit as f64).ceil() as u64,
            }
        }),
    );
    diff.0
        .retain(|op| matches!(op, PatchOperation::Add(AddOperation { .. })));

    diff.0.push(PatchOperation::Add(json_patch::AddOperation {
        path: Pointer::new(["pagination", "previous"]),
        value: if info.page > 1 {
            serde_json::to_value(info.page - 1).unwrap()
        } else {
            serde_json::Value::Null
        },
    }));
    diff.0.push(PatchOperation::Add(json_patch::AddOperation {
        path: Pointer::new(["pagination", "next"]),
        value: if info.page * info.limit < info.count {
            serde_json::to_value(info.page + 1).unwrap()
        } else {
            serde_json::Value::Null
        },
    }));

    json_patch::patch(target, &diff).unwrap();
}
