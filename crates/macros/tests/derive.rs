use adrastos_macros::DbDeserialize;

#[test]
fn test() {
    #[derive(DbDeserialize)]
    struct Test {
        _a: Option<i32>,
        _b: String,
    }
}
