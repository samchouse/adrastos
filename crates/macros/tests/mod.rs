use macros::EntityBuilder;

#[test]
fn it_works() {
    #[derive(EntityBuilder)]
    struct User;

    User::select().finish();
}
