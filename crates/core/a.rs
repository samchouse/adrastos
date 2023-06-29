pub mod user {
    use adrastos_macros::{DbCommon, DbQuery, DbSelect};
    use chrono::{DateTime, Utc};
    use sea_query::{enum_def, Alias, Expr, PostgresQueryBuilder};
    use serde::{Deserialize, Serialize};
    use tracing::error;
    use tracing_unwrap::ResultExt;
    use utoipa::ToSchema;
    use validator::Validate;
    use crate::{auth, error::Error};
    use super::{Connection, Identity, RefreshTokenTree, Update};
    fn validate_password(password: String) -> Result<String, Error> {
        auth::hash_password(&password)
            .map_err(|err| {
                Error::InternalServerError({
                    let res = ::alloc::fmt::format(
                        format_args!(
                            "An error occurred while hashing the password for the {0}",
                            err,
                        ),
                    );
                    res
                })
            })
    }
    #[adrastos(validated)]
    #[serde(rename_all = "camelCase")]
    pub struct User {
        pub id: String,
        #[adrastos(find)]
        #[validate(length(max = 50))]
        pub first_name: String,
        #[adrastos(find)]
        #[validate(length(max = 50))]
        pub last_name: String,
        #[adrastos(find, unique)]
        #[validate(email)]
        pub email: String,
        #[adrastos(find, unique)]
        #[validate(length(min = 5, max = 64))]
        pub username: String,
        #[serde(skip_serializing)]
        #[adrastos(transform = validate_password)]
        #[validate(length(min = 8, max = 64))]
        pub password: String,
        pub verified: bool,
        pub banned: bool,
        #[serde(skip_serializing)]
        pub mfa_secret: Option<String>,
        #[serde(skip_serializing)]
        pub mfa_backup_codes: Option<Vec<String>>,
        pub created_at: DateTime<Utc>,
        pub updated_at: Option<DateTime<Utc>>,
        #[adrastos(join)]
        #[serde(skip_serializing)]
        pub connections: Option<Vec<Connection>>,
        #[adrastos(join)]
        #[serde(skip_serializing)]
        pub refresh_token_trees: Option<Vec<RefreshTokenTree>>,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for User {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "id",
                "first_name",
                "last_name",
                "email",
                "username",
                "password",
                "verified",
                "banned",
                "mfa_secret",
                "mfa_backup_codes",
                "created_at",
                "updated_at",
                "connections",
                "refresh_token_trees",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.id,
                &self.first_name,
                &self.last_name,
                &self.email,
                &self.username,
                &self.password,
                &self.verified,
                &self.banned,
                &self.mfa_secret,
                &self.mfa_backup_codes,
                &self.created_at,
                &self.updated_at,
                &self.connections,
                &&self.refresh_token_trees,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(f, "User", names, values)
        }
    }
    impl ::validator::Validate for User {
        fn validate(&self) -> ::std::result::Result<(), ::validator::ValidationErrors> {
            use ::validator::ValidateArgs;
            self.validate_args(())
        }
    }
    #[allow(clippy::all)]
    #[allow(single_use_lifetimes)]
    impl<'v_a> ::validator::ValidateArgs<'v_a> for User {
        type Args = ();
        #[allow(unused_mut)]
        #[allow(unused_variable)]
        fn validate_args(
            &self,
            args: Self::Args,
        ) -> ::std::result::Result<(), ::validator::ValidationErrors> {
            let mut errors = ::validator::ValidationErrors::new();
            if !::validator::validate_length(
                &self.first_name,
                ::std::option::Option::None,
                ::std::option::Option::Some(50u64 as u64),
                ::std::option::Option::None,
            ) {
                let mut err = ::validator::ValidationError::new("length");
                err.add_param(::std::borrow::Cow::from("max"), &50u64);
                err.add_param(::std::borrow::Cow::from("value"), &&self.first_name);
                errors.add("first_name", err);
            }
            if !::validator::validate_length(
                &self.last_name,
                ::std::option::Option::None,
                ::std::option::Option::Some(50u64 as u64),
                ::std::option::Option::None,
            ) {
                let mut err = ::validator::ValidationError::new("length");
                err.add_param(::std::borrow::Cow::from("max"), &50u64);
                err.add_param(::std::borrow::Cow::from("value"), &&self.last_name);
                errors.add("last_name", err);
            }
            if !::validator::validate_email(&self.email) {
                let mut err = ::validator::ValidationError::new("email");
                err.add_param(::std::borrow::Cow::from("value"), &&self.email);
                errors.add("email", err);
            }
            if !::validator::validate_length(
                &self.username,
                ::std::option::Option::Some(5u64 as u64),
                ::std::option::Option::Some(64u64 as u64),
                ::std::option::Option::None,
            ) {
                let mut err = ::validator::ValidationError::new("length");
                err.add_param(::std::borrow::Cow::from("min"), &5u64);
                err.add_param(::std::borrow::Cow::from("max"), &64u64);
                err.add_param(::std::borrow::Cow::from("value"), &&self.username);
                errors.add("username", err);
            }
            if !::validator::validate_length(
                &self.password,
                ::std::option::Option::Some(8u64 as u64),
                ::std::option::Option::Some(64u64 as u64),
                ::std::option::Option::None,
            ) {
                let mut err = ::validator::ValidationError::new("length");
                err.add_param(::std::borrow::Cow::from("min"), &8u64);
                err.add_param(::std::borrow::Cow::from("max"), &64u64);
                err.add_param(::std::borrow::Cow::from("value"), &&self.password);
                errors.add("password", err);
            }
            let mut result = if errors.is_empty() {
                ::std::result::Result::Ok(())
            } else {
                ::std::result::Result::Err(errors)
            };
            result
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for User {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = match _serde::Serializer::serialize_struct(
                    __serializer,
                    "User",
                    false as usize + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "id",
                    &self.id,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "firstName",
                    &self.first_name,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "lastName",
                    &self.last_name,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "email",
                    &self.email,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "username",
                    &self.username,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "verified",
                    &self.verified,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "banned",
                    &self.banned,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "createdAt",
                    &self.created_at,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "updatedAt",
                    &self.updated_at,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for User {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                    __field4,
                    __field5,
                    __field6,
                    __field7,
                    __field8,
                    __field9,
                    __field10,
                    __field11,
                    __field12,
                    __field13,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private::Ok(__Field::__field0),
                            1u64 => _serde::__private::Ok(__Field::__field1),
                            2u64 => _serde::__private::Ok(__Field::__field2),
                            3u64 => _serde::__private::Ok(__Field::__field3),
                            4u64 => _serde::__private::Ok(__Field::__field4),
                            5u64 => _serde::__private::Ok(__Field::__field5),
                            6u64 => _serde::__private::Ok(__Field::__field6),
                            7u64 => _serde::__private::Ok(__Field::__field7),
                            8u64 => _serde::__private::Ok(__Field::__field8),
                            9u64 => _serde::__private::Ok(__Field::__field9),
                            10u64 => _serde::__private::Ok(__Field::__field10),
                            11u64 => _serde::__private::Ok(__Field::__field11),
                            12u64 => _serde::__private::Ok(__Field::__field12),
                            13u64 => _serde::__private::Ok(__Field::__field13),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "id" => _serde::__private::Ok(__Field::__field0),
                            "firstName" => _serde::__private::Ok(__Field::__field1),
                            "lastName" => _serde::__private::Ok(__Field::__field2),
                            "email" => _serde::__private::Ok(__Field::__field3),
                            "username" => _serde::__private::Ok(__Field::__field4),
                            "password" => _serde::__private::Ok(__Field::__field5),
                            "verified" => _serde::__private::Ok(__Field::__field6),
                            "banned" => _serde::__private::Ok(__Field::__field7),
                            "mfaSecret" => _serde::__private::Ok(__Field::__field8),
                            "mfaBackupCodes" => _serde::__private::Ok(__Field::__field9),
                            "createdAt" => _serde::__private::Ok(__Field::__field10),
                            "updatedAt" => _serde::__private::Ok(__Field::__field11),
                            "connections" => _serde::__private::Ok(__Field::__field12),
                            "refreshTokenTrees" => {
                                _serde::__private::Ok(__Field::__field13)
                            }
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"id" => _serde::__private::Ok(__Field::__field0),
                            b"firstName" => _serde::__private::Ok(__Field::__field1),
                            b"lastName" => _serde::__private::Ok(__Field::__field2),
                            b"email" => _serde::__private::Ok(__Field::__field3),
                            b"username" => _serde::__private::Ok(__Field::__field4),
                            b"password" => _serde::__private::Ok(__Field::__field5),
                            b"verified" => _serde::__private::Ok(__Field::__field6),
                            b"banned" => _serde::__private::Ok(__Field::__field7),
                            b"mfaSecret" => _serde::__private::Ok(__Field::__field8),
                            b"mfaBackupCodes" => _serde::__private::Ok(__Field::__field9),
                            b"createdAt" => _serde::__private::Ok(__Field::__field10),
                            b"updatedAt" => _serde::__private::Ok(__Field::__field11),
                            b"connections" => _serde::__private::Ok(__Field::__field12),
                            b"refreshTokenTrees" => {
                                _serde::__private::Ok(__Field::__field13)
                            }
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                }
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<User>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = User;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "struct User",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        } {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct User with 14 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        } {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct User with 14 elements",
                                    ),
                                );
                            }
                        };
                        let __field2 = match match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        } {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        2usize,
                                        &"struct User with 14 elements",
                                    ),
                                );
                            }
                        };
                        let __field3 = match match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        } {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        3usize,
                                        &"struct User with 14 elements",
                                    ),
                                );
                            }
                        };
                        let __field4 = match match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        } {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        4usize,
                                        &"struct User with 14 elements",
                                    ),
                                );
                            }
                        };
                        let __field5 = match match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        } {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        5usize,
                                        &"struct User with 14 elements",
                                    ),
                                );
                            }
                        };
                        let __field6 = match match _serde::de::SeqAccess::next_element::<
                            bool,
                        >(&mut __seq) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        } {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        6usize,
                                        &"struct User with 14 elements",
                                    ),
                                );
                            }
                        };
                        let __field7 = match match _serde::de::SeqAccess::next_element::<
                            bool,
                        >(&mut __seq) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        } {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        7usize,
                                        &"struct User with 14 elements",
                                    ),
                                );
                            }
                        };
                        let __field8 = match match _serde::de::SeqAccess::next_element::<
                            Option<String>,
                        >(&mut __seq) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        } {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        8usize,
                                        &"struct User with 14 elements",
                                    ),
                                );
                            }
                        };
                        let __field9 = match match _serde::de::SeqAccess::next_element::<
                            Option<Vec<String>>,
                        >(&mut __seq) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        } {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        9usize,
                                        &"struct User with 14 elements",
                                    ),
                                );
                            }
                        };
                        let __field10 = match match _serde::de::SeqAccess::next_element::<
                            DateTime<Utc>,
                        >(&mut __seq) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        } {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        10usize,
                                        &"struct User with 14 elements",
                                    ),
                                );
                            }
                        };
                        let __field11 = match match _serde::de::SeqAccess::next_element::<
                            Option<DateTime<Utc>>,
                        >(&mut __seq) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        } {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        11usize,
                                        &"struct User with 14 elements",
                                    ),
                                );
                            }
                        };
                        let __field12 = match match _serde::de::SeqAccess::next_element::<
                            Option<Vec<Connection>>,
                        >(&mut __seq) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        } {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        12usize,
                                        &"struct User with 14 elements",
                                    ),
                                );
                            }
                        };
                        let __field13 = match match _serde::de::SeqAccess::next_element::<
                            Option<Vec<RefreshTokenTree>>,
                        >(&mut __seq) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        } {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        13usize,
                                        &"struct User with 14 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private::Ok(User {
                            id: __field0,
                            first_name: __field1,
                            last_name: __field2,
                            email: __field3,
                            username: __field4,
                            password: __field5,
                            verified: __field6,
                            banned: __field7,
                            mfa_secret: __field8,
                            mfa_backup_codes: __field9,
                            created_at: __field10,
                            updated_at: __field11,
                            connections: __field12,
                            refresh_token_trees: __field13,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private::Option<String> = _serde::__private::None;
                        let mut __field1: _serde::__private::Option<String> = _serde::__private::None;
                        let mut __field2: _serde::__private::Option<String> = _serde::__private::None;
                        let mut __field3: _serde::__private::Option<String> = _serde::__private::None;
                        let mut __field4: _serde::__private::Option<String> = _serde::__private::None;
                        let mut __field5: _serde::__private::Option<String> = _serde::__private::None;
                        let mut __field6: _serde::__private::Option<bool> = _serde::__private::None;
                        let mut __field7: _serde::__private::Option<bool> = _serde::__private::None;
                        let mut __field8: _serde::__private::Option<Option<String>> = _serde::__private::None;
                        let mut __field9: _serde::__private::Option<
                            Option<Vec<String>>,
                        > = _serde::__private::None;
                        let mut __field10: _serde::__private::Option<DateTime<Utc>> = _serde::__private::None;
                        let mut __field11: _serde::__private::Option<
                            Option<DateTime<Utc>>,
                        > = _serde::__private::None;
                        let mut __field12: _serde::__private::Option<
                            Option<Vec<Connection>>,
                        > = _serde::__private::None;
                        let mut __field13: _serde::__private::Option<
                            Option<Vec<RefreshTokenTree>>,
                        > = _serde::__private::None;
                        while let _serde::__private::Some(__key)
                            = match _serde::de::MapAccess::next_key::<
                                __Field,
                            >(&mut __map) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private::Option::is_some(&__field0) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("id"),
                                        );
                                    }
                                    __field0 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<
                                            String,
                                        >(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private::Option::is_some(&__field1) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "firstName",
                                            ),
                                        );
                                    }
                                    __field1 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<
                                            String,
                                        >(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private::Option::is_some(&__field2) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "lastName",
                                            ),
                                        );
                                    }
                                    __field2 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<
                                            String,
                                        >(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field3 => {
                                    if _serde::__private::Option::is_some(&__field3) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("email"),
                                        );
                                    }
                                    __field3 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<
                                            String,
                                        >(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field4 => {
                                    if _serde::__private::Option::is_some(&__field4) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "username",
                                            ),
                                        );
                                    }
                                    __field4 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<
                                            String,
                                        >(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field5 => {
                                    if _serde::__private::Option::is_some(&__field5) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "password",
                                            ),
                                        );
                                    }
                                    __field5 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<
                                            String,
                                        >(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field6 => {
                                    if _serde::__private::Option::is_some(&__field6) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "verified",
                                            ),
                                        );
                                    }
                                    __field6 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<
                                            bool,
                                        >(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field7 => {
                                    if _serde::__private::Option::is_some(&__field7) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("banned"),
                                        );
                                    }
                                    __field7 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<
                                            bool,
                                        >(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field8 => {
                                    if _serde::__private::Option::is_some(&__field8) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "mfaSecret",
                                            ),
                                        );
                                    }
                                    __field8 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<
                                            Option<String>,
                                        >(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field9 => {
                                    if _serde::__private::Option::is_some(&__field9) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "mfaBackupCodes",
                                            ),
                                        );
                                    }
                                    __field9 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<
                                            Option<Vec<String>>,
                                        >(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field10 => {
                                    if _serde::__private::Option::is_some(&__field10) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "createdAt",
                                            ),
                                        );
                                    }
                                    __field10 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<
                                            DateTime<Utc>,
                                        >(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field11 => {
                                    if _serde::__private::Option::is_some(&__field11) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "updatedAt",
                                            ),
                                        );
                                    }
                                    __field11 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<
                                            Option<DateTime<Utc>>,
                                        >(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field12 => {
                                    if _serde::__private::Option::is_some(&__field12) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "connections",
                                            ),
                                        );
                                    }
                                    __field12 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<
                                            Option<Vec<Connection>>,
                                        >(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field13 => {
                                    if _serde::__private::Option::is_some(&__field13) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "refreshTokenTrees",
                                            ),
                                        );
                                    }
                                    __field13 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<
                                            Option<Vec<RefreshTokenTree>>,
                                        >(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                _ => {
                                    let _ = match _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    };
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private::Some(__field0) => __field0,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("id") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private::Some(__field1) => __field1,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("firstName") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private::Some(__field2) => __field2,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("lastName") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field3 = match __field3 {
                            _serde::__private::Some(__field3) => __field3,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("email") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field4 = match __field4 {
                            _serde::__private::Some(__field4) => __field4,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("username") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field5 = match __field5 {
                            _serde::__private::Some(__field5) => __field5,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("password") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field6 = match __field6 {
                            _serde::__private::Some(__field6) => __field6,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("verified") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field7 = match __field7 {
                            _serde::__private::Some(__field7) => __field7,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("banned") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field8 = match __field8 {
                            _serde::__private::Some(__field8) => __field8,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("mfaSecret") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field9 = match __field9 {
                            _serde::__private::Some(__field9) => __field9,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field(
                                    "mfaBackupCodes",
                                ) {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field10 = match __field10 {
                            _serde::__private::Some(__field10) => __field10,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("createdAt") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field11 = match __field11 {
                            _serde::__private::Some(__field11) => __field11,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("updatedAt") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field12 = match __field12 {
                            _serde::__private::Some(__field12) => __field12,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("connections") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field13 = match __field13 {
                            _serde::__private::Some(__field13) => __field13,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field(
                                    "refreshTokenTrees",
                                ) {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        _serde::__private::Ok(User {
                            id: __field0,
                            first_name: __field1,
                            last_name: __field2,
                            email: __field3,
                            username: __field4,
                            password: __field5,
                            verified: __field6,
                            banned: __field7,
                            mfa_secret: __field8,
                            mfa_backup_codes: __field9,
                            created_at: __field10,
                            updated_at: __field11,
                            connections: __field12,
                            refresh_token_trees: __field13,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
                    "id",
                    "firstName",
                    "lastName",
                    "email",
                    "username",
                    "password",
                    "verified",
                    "banned",
                    "mfaSecret",
                    "mfaBackupCodes",
                    "createdAt",
                    "updatedAt",
                    "connections",
                    "refreshTokenTrees",
                ];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "User",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<User>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for User {
        #[inline]
        fn clone(&self) -> User {
            User {
                id: ::core::clone::Clone::clone(&self.id),
                first_name: ::core::clone::Clone::clone(&self.first_name),
                last_name: ::core::clone::Clone::clone(&self.last_name),
                email: ::core::clone::Clone::clone(&self.email),
                username: ::core::clone::Clone::clone(&self.username),
                password: ::core::clone::Clone::clone(&self.password),
                verified: ::core::clone::Clone::clone(&self.verified),
                banned: ::core::clone::Clone::clone(&self.banned),
                mfa_secret: ::core::clone::Clone::clone(&self.mfa_secret),
                mfa_backup_codes: ::core::clone::Clone::clone(&self.mfa_backup_codes),
                created_at: ::core::clone::Clone::clone(&self.created_at),
                updated_at: ::core::clone::Clone::clone(&self.updated_at),
                connections: ::core::clone::Clone::clone(&self.connections),
                refresh_token_trees: ::core::clone::Clone::clone(
                    &self.refresh_token_trees,
                ),
            }
        }
    }
    impl<'__s> utoipa::ToSchema<'__s> for User {
        fn schema() -> (
            &'__s str,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        ) {
            (
                "User",
                utoipa::openapi::ObjectBuilder::new()
                    .property(
                        "id",
                        utoipa::openapi::ObjectBuilder::new()
                            .schema_type(utoipa::openapi::SchemaType::String),
                    )
                    .required("id")
                    .property(
                        "firstName",
                        utoipa::openapi::ObjectBuilder::new()
                            .schema_type(utoipa::openapi::SchemaType::String),
                    )
                    .required("firstName")
                    .property(
                        "lastName",
                        utoipa::openapi::ObjectBuilder::new()
                            .schema_type(utoipa::openapi::SchemaType::String),
                    )
                    .required("lastName")
                    .property(
                        "email",
                        utoipa::openapi::ObjectBuilder::new()
                            .schema_type(utoipa::openapi::SchemaType::String),
                    )
                    .required("email")
                    .property(
                        "username",
                        utoipa::openapi::ObjectBuilder::new()
                            .schema_type(utoipa::openapi::SchemaType::String),
                    )
                    .required("username")
                    .property(
                        "verified",
                        utoipa::openapi::ObjectBuilder::new()
                            .schema_type(utoipa::openapi::SchemaType::Boolean),
                    )
                    .required("verified")
                    .property(
                        "banned",
                        utoipa::openapi::ObjectBuilder::new()
                            .schema_type(utoipa::openapi::SchemaType::Boolean),
                    )
                    .required("banned")
                    .property(
                        "createdAt",
                        utoipa::openapi::ObjectBuilder::new()
                            .schema_type(utoipa::openapi::SchemaType::String)
                            .format(
                                Some(
                                    utoipa::openapi::SchemaFormat::KnownFormat(
                                        utoipa::openapi::KnownFormat::DateTime,
                                    ),
                                ),
                            ),
                    )
                    .required("createdAt")
                    .property(
                        "updatedAt",
                        utoipa::openapi::ObjectBuilder::new()
                            .schema_type(utoipa::openapi::SchemaType::String)
                            .format(
                                Some(
                                    utoipa::openapi::SchemaFormat::KnownFormat(
                                        utoipa::openapi::KnownFormat::DateTime,
                                    ),
                                ),
                            )
                            .nullable(true),
                    )
                    .into(),
            )
        }
    }
    impl User {
        pub fn init() -> String {
            sea_query::Table::create()
                .table(Self::table())
                .if_not_exists()
                .col(
                    sea_query::ColumnDef::new(sea_query::Alias::new("id"))
                        .string()
                        .not_null()
                        .primary_key(),
                )
                .col(
                    sea_query::ColumnDef::new(sea_query::Alias::new("first_name"))
                        .string()
                        .not_null(),
                )
                .col(
                    sea_query::ColumnDef::new(sea_query::Alias::new("last_name"))
                        .string()
                        .not_null(),
                )
                .col(
                    sea_query::ColumnDef::new(sea_query::Alias::new("email"))
                        .string()
                        .not_null()
                        .unique_key(),
                )
                .col(
                    sea_query::ColumnDef::new(sea_query::Alias::new("username"))
                        .string()
                        .not_null()
                        .unique_key(),
                )
                .col(
                    sea_query::ColumnDef::new(sea_query::Alias::new("password"))
                        .string()
                        .not_null(),
                )
                .col(
                    sea_query::ColumnDef::new(sea_query::Alias::new("verified"))
                        .boolean()
                        .not_null()
                        .default(false),
                )
                .col(
                    sea_query::ColumnDef::new(sea_query::Alias::new("banned"))
                        .boolean()
                        .not_null()
                        .default(false),
                )
                .col(
                    sea_query::ColumnDef::new(sea_query::Alias::new("mfa_secret"))
                        .string(),
                )
                .col(
                    sea_query::ColumnDef::new(sea_query::Alias::new("mfa_backup_codes"))
                        .array(sea_query::ColumnType::String(None)),
                )
                .col(
                    sea_query::ColumnDef::new(sea_query::Alias::new("created_at"))
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(sea_query::Keyword::CurrentTimestamp),
                )
                .col(
                    sea_query::ColumnDef::new(sea_query::Alias::new("updated_at"))
                        .timestamp_with_time_zone(),
                )
                .to_string(sea_query::PostgresQueryBuilder)
        }
    }
    impl Identity for User {
        fn table() -> sea_query::Alias {
            sea_query::Alias::new(UserIden::Table.to_string())
        }
        fn error_identifier() -> String {
            "user".to_string()
        }
    }
    impl std::fmt::Display for UserIden {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let name = match self {
                Self::Table => "users",
                Self::Id => "id",
                Self::FirstName => "first_name",
                Self::LastName => "last_name",
                Self::Email => "email",
                Self::Username => "username",
                Self::Password => "password",
                Self::Verified => "verified",
                Self::Banned => "banned",
                Self::MfaSecret => "mfa_secret",
                Self::MfaBackupCodes => "mfa_backup_codes",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
                Self::Connections => "connections",
                Self::RefreshTokenTrees => "refresh_token_trees",
            };
            f.write_fmt(format_args!("{0}", name))
        }
    }
    impl From<deadpool_postgres::tokio_postgres::Row> for User {
        fn from(row: deadpool_postgres::tokio_postgres::Row) -> Self {
            User {
                id: row.get("id"),
                first_name: row.get("first_name"),
                last_name: row.get("last_name"),
                email: row.get("email"),
                username: row.get("username"),
                password: row.get("password"),
                verified: row.get("verified"),
                banned: row.get("banned"),
                mfa_secret: row.get("mfa_secret"),
                mfa_backup_codes: row.get("mfa_backup_codes"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                connections: row
                    .try_get::<_, Option<Vec<String>>>("connections")
                    .ok()
                    .flatten()
                    .map(|v| {
                        v
                            .iter()
                            .map(|s| serde_json::from_str(&s).unwrap())
                            .collect::<Vec<_>>()
                    }),
                refresh_token_trees: row
                    .try_get::<_, Option<Vec<String>>>("refresh_token_trees")
                    .ok()
                    .flatten()
                    .map(|v| {
                        v
                            .iter()
                            .map(|s| serde_json::from_str(&s).unwrap())
                            .collect::<Vec<_>>()
                    }),
            }
        }
    }
    pub struct UserSelectBuilder {
        query_builder: sea_query::SelectStatement,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UserSelectBuilder {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "UserSelectBuilder",
                "query_builder",
                &&self.query_builder,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for UserSelectBuilder {
        #[inline]
        fn clone(&self) -> UserSelectBuilder {
            UserSelectBuilder {
                query_builder: ::core::clone::Clone::clone(&self.query_builder),
            }
        }
    }
    impl UserSelectBuilder {
        fn by_id(&mut self, id: &str) -> &mut Self {
            self.query_builder.and_where(sea_query::Expr::col(UserIden::Id).eq(id));
            self
        }
        async fn finish(
            &mut self,
            db_pool: &deadpool_postgres::Pool,
        ) -> Result<User, crate::error::Error> {
            let row = db_pool
                .get()
                .await
                .unwrap()
                .query(self.to_string().as_str(), &[])
                .await
                .map_err(|e| {
                    let error = {
                        let res = ::alloc::fmt::format(
                            format_args!(
                                "An error occurred while fetching the {0}: {1}",
                                User::error_identifier(),
                                e,
                            ),
                        );
                        res
                    };
                    crate::error::Error::InternalServerError(error)
                })?
                .into_iter()
                .next()
                .ok_or_else(|| {
                    let message = {
                        let res = ::alloc::fmt::format(
                            format_args!("No {0} was found", User::error_identifier()),
                        );
                        res
                    };
                    crate::error::Error::BadRequest(message)
                })?;
            Ok(row.into())
        }
        pub fn by_first_name(&mut self, first_name: String) -> &mut Self {
            self.query_builder
                .and_where(Expr::col(Alias::new("first_name")).eq(first_name));
            self
        }
        pub fn by_last_name(&mut self, last_name: String) -> &mut Self {
            self.query_builder
                .and_where(Expr::col(Alias::new("last_name")).eq(last_name));
            self
        }
        pub fn by_email(&mut self, email: String) -> &mut Self {
            self.query_builder.and_where(Expr::col(Alias::new("email")).eq(email));
            self
        }
        pub fn by_username(&mut self, username: String) -> &mut Self {
            self.query_builder.and_where(Expr::col(Alias::new("username")).eq(username));
            self
        }
        pub fn and_where(
            &mut self,
            expressions: Vec<sea_query::SimpleExpr>,
        ) -> &mut Self {
            for expression in expressions {
                self.query_builder.and_where(expression);
            }
            self
        }
        pub fn join(&mut self, join: UserJoin) -> &mut Self {
            let query = match join {
                UserJoin::Connections => {
                    Connection::find()
                        .and_where(
                            <[_]>::into_vec(
                                #[rustc_box]
                                ::alloc::boxed::Box::new([
                                    sea_query::Expr::col(sea_query::Alias::new("connections"))
                                        .equals((Connection::table(), sea_query::Alias::new("id"))),
                                ]),
                            ),
                        )
                        .to_string()
                }
                UserJoin::RefreshTokenTrees => {
                    RefreshTokenTree::find()
                        .and_where(
                            <[_]>::into_vec(
                                #[rustc_box]
                                ::alloc::boxed::Box::new([
                                    sea_query::Expr::col(
                                            sea_query::Alias::new("refresh_token_trees"),
                                        )
                                        .equals((
                                            RefreshTokenTree::table(),
                                            sea_query::Alias::new("id"),
                                        )),
                                ]),
                            ),
                        )
                        .to_string()
                }
            };
            self.query_builder
                .expr(
                    sea_query::Expr::cust(
                        {
                            let res = ::alloc::fmt::format(
                                format_args!(
                                    "(SELECT json_agg({0}) FROM ({3}) {1}) as {2}",
                                    join.to_string(),
                                    join.to_string(),
                                    {
                                        let res = ::alloc::fmt::format(
                                            format_args!("{0}s", join.to_string()),
                                        );
                                        res
                                    },
                                    query,
                                ),
                            );
                            res
                        }
                            .as_str(),
                    ),
                );
            self
        }
        pub async fn one(
            &mut self,
            db_pool: &deadpool_postgres::Pool,
        ) -> Result<User, crate::error::Error> {
            self.query_builder.reset_limit().limit(1);
            self.finish(db_pool).await
        }
        pub async fn all(
            &mut self,
            db_pool: &deadpool_postgres::Pool,
        ) -> Result<User, crate::error::Error> {
            self.query_builder.reset_limit();
            self.finish(db_pool).await
        }
        pub fn to_string(&self) -> String {
            self.query_builder.to_string(sea_query::PostgresQueryBuilder)
        }
    }
    impl User {
        pub fn find() -> UserSelectBuilder {
            UserSelectBuilder {
                query_builder: sea_query::Query::select()
                    .from(Self::table())
                    .columns([
                        Alias::new("id"),
                        Alias::new("first_name"),
                        Alias::new("last_name"),
                        Alias::new("email"),
                        Alias::new("username"),
                        Alias::new("password"),
                        Alias::new("verified"),
                        Alias::new("banned"),
                        Alias::new("mfa_secret"),
                        Alias::new("mfa_backup_codes"),
                        Alias::new("created_at"),
                        Alias::new("updated_at"),
                    ])
                    .to_owned(),
            }
        }
        pub fn find_by_id(id: &str) -> UserSelectBuilder {
            let mut builder = Self::find();
            builder.by_id(id).to_owned()
        }
    }
    impl User {
        pub async fn create(
            &self,
            db_pool: &deadpool_postgres::Pool,
        ) -> Result<(), crate::error::Error> {
            self.validate()
                .map_err(|err| crate::error::Error::ValidationErrors {
                    message: {
                        let res = ::alloc::fmt::format(
                            format_args!(
                                "An error occurred while validating the {0}",
                                Self::error_identifier(),
                            ),
                        );
                        res
                    },
                    errors: err,
                })?;
            let query = sea_query::Query::insert()
                .into_table(Self::table())
                .columns([
                    sea_query::Alias::new("id"),
                    sea_query::Alias::new("first_name"),
                    sea_query::Alias::new("last_name"),
                    sea_query::Alias::new("email"),
                    sea_query::Alias::new("username"),
                    sea_query::Alias::new("password"),
                    sea_query::Alias::new("verified"),
                    sea_query::Alias::new("banned"),
                    sea_query::Alias::new("mfa_secret"),
                    sea_query::Alias::new("mfa_backup_codes"),
                    sea_query::Alias::new("created_at"),
                    sea_query::Alias::new("updated_at"),
                ])
                .values_panic([
                    self.id.clone().into(),
                    self.first_name.clone().into(),
                    self.last_name.clone().into(),
                    self.email.clone().into(),
                    self.username.clone().into(),
                    validate_password(self.password.clone().into())?.into(),
                    self.verified.into(),
                    self.banned.into(),
                    self.mfa_secret.clone().into(),
                    self.mfa_backup_codes.clone().into(),
                    self.created_at.into(),
                    self.updated_at.into(),
                ])
                .to_string(sea_query::PostgresQueryBuilder);
            db_pool
                .get()
                .await
                .unwrap()
                .execute(&query, &[])
                .await
                .map_err(|e| {
                    {
                        use ::tracing::__macro_support::Callsite as _;
                        static CALLSITE: ::tracing::callsite::DefaultCallsite = {
                            static META: ::tracing::Metadata<'static> = {
                                ::tracing_core::metadata::Metadata::new(
                                    "event crates/core/src/entities/user.rs:23",
                                    "adrastos_core::entities::user",
                                    ::tracing::Level::ERROR,
                                    Some("crates/core/src/entities/user.rs"),
                                    Some(23u32),
                                    Some("adrastos_core::entities::user"),
                                    ::tracing_core::field::FieldSet::new(
                                        &["error"],
                                        ::tracing_core::callsite::Identifier(&CALLSITE),
                                    ),
                                    ::tracing::metadata::Kind::EVENT,
                                )
                            };
                            ::tracing::callsite::DefaultCallsite::new(&META)
                        };
                        let enabled = ::tracing::Level::ERROR
                            <= ::tracing::level_filters::STATIC_MAX_LEVEL
                            && ::tracing::Level::ERROR
                                <= ::tracing::level_filters::LevelFilter::current()
                            && {
                                let interest = CALLSITE.interest();
                                !interest.is_never()
                                    && ::tracing::__macro_support::__is_enabled(
                                        CALLSITE.metadata(),
                                        interest,
                                    )
                            };
                        if enabled {
                            (|value_set: ::tracing::field::ValueSet| {
                                let meta = CALLSITE.metadata();
                                ::tracing::Event::dispatch(meta, &value_set);
                                if (match ::tracing::Level::ERROR {
                                    ::tracing::Level::ERROR => ::tracing::log::Level::Error,
                                    ::tracing::Level::WARN => ::tracing::log::Level::Warn,
                                    ::tracing::Level::INFO => ::tracing::log::Level::Info,
                                    ::tracing::Level::DEBUG => ::tracing::log::Level::Debug,
                                    _ => ::tracing::log::Level::Trace,
                                }) <= ::tracing::log::STATIC_MAX_LEVEL
                                {
                                    if !::tracing::dispatcher::has_been_set() {
                                        {
                                            use ::tracing::log;
                                            let level = match ::tracing::Level::ERROR {
                                                ::tracing::Level::ERROR => ::tracing::log::Level::Error,
                                                ::tracing::Level::WARN => ::tracing::log::Level::Warn,
                                                ::tracing::Level::INFO => ::tracing::log::Level::Info,
                                                ::tracing::Level::DEBUG => ::tracing::log::Level::Debug,
                                                _ => ::tracing::log::Level::Trace,
                                            };
                                            if level <= log::max_level() {
                                                let meta = CALLSITE.metadata();
                                                let log_meta = log::Metadata::builder()
                                                    .level(level)
                                                    .target(meta.target())
                                                    .build();
                                                let logger = log::logger();
                                                if logger.enabled(&log_meta) {
                                                    ::tracing::__macro_support::__tracing_log(
                                                        meta,
                                                        logger,
                                                        log_meta,
                                                        &value_set,
                                                    )
                                                }
                                            }
                                        }
                                    } else {
                                        {}
                                    }
                                } else {
                                    {}
                                };
                            })({
                                #[allow(unused_imports)]
                                use ::tracing::field::{debug, display, Value};
                                let mut iter = CALLSITE.metadata().fields().iter();
                                CALLSITE
                                    .metadata()
                                    .fields()
                                    .value_set(
                                        &[
                                            (
                                                &iter.next().expect("FieldSet corrupted (this is a bug)"),
                                                Some(&debug(&e) as &dyn Value),
                                            ),
                                        ],
                                    )
                            });
                        } else {
                            if (match ::tracing::Level::ERROR {
                                ::tracing::Level::ERROR => ::tracing::log::Level::Error,
                                ::tracing::Level::WARN => ::tracing::log::Level::Warn,
                                ::tracing::Level::INFO => ::tracing::log::Level::Info,
                                ::tracing::Level::DEBUG => ::tracing::log::Level::Debug,
                                _ => ::tracing::log::Level::Trace,
                            }) <= ::tracing::log::STATIC_MAX_LEVEL
                            {
                                if !::tracing::dispatcher::has_been_set() {
                                    {
                                        use ::tracing::log;
                                        let level = match ::tracing::Level::ERROR {
                                            ::tracing::Level::ERROR => ::tracing::log::Level::Error,
                                            ::tracing::Level::WARN => ::tracing::log::Level::Warn,
                                            ::tracing::Level::INFO => ::tracing::log::Level::Info,
                                            ::tracing::Level::DEBUG => ::tracing::log::Level::Debug,
                                            _ => ::tracing::log::Level::Trace,
                                        };
                                        if level <= log::max_level() {
                                            let meta = CALLSITE.metadata();
                                            let log_meta = log::Metadata::builder()
                                                .level(level)
                                                .target(meta.target())
                                                .build();
                                            let logger = log::logger();
                                            if logger.enabled(&log_meta) {
                                                ::tracing::__macro_support::__tracing_log(
                                                    meta,
                                                    logger,
                                                    log_meta,
                                                    &{
                                                        #[allow(unused_imports)]
                                                        use ::tracing::field::{debug, display, Value};
                                                        let mut iter = CALLSITE.metadata().fields().iter();
                                                        CALLSITE
                                                            .metadata()
                                                            .fields()
                                                            .value_set(
                                                                &[
                                                                    (
                                                                        &iter.next().expect("FieldSet corrupted (this is a bug)"),
                                                                        Some(&debug(&e) as &dyn Value),
                                                                    ),
                                                                ],
                                                            )
                                                    },
                                                )
                                            }
                                        }
                                    }
                                } else {
                                    {}
                                }
                            } else {
                                {}
                            };
                        }
                    };
                    crate::error::Error::InternalServerError({
                        let res = ::alloc::fmt::format(
                            format_args!(
                                "Failed to create {0}",
                                Self::error_identifier(),
                            ),
                        );
                        res
                    })
                })?;
            Ok(())
        }
        pub async fn delete(
            &self,
            db_pool: &deadpool_postgres::Pool,
        ) -> Result<(), crate::error::Error> {
            let query = sea_query::Query::delete()
                .from_table(Self::table())
                .and_where(
                    sea_query::Expr::col(sea_query::Alias::new("id")).eq(self.id.clone()),
                )
                .to_string(sea_query::PostgresQueryBuilder);
            db_pool
                .get()
                .await
                .unwrap()
                .execute(&query, &[])
                .await
                .map_err(|e| {
                    {
                        use ::tracing::__macro_support::Callsite as _;
                        static CALLSITE: ::tracing::callsite::DefaultCallsite = {
                            static META: ::tracing::Metadata<'static> = {
                                ::tracing_core::metadata::Metadata::new(
                                    "event crates/core/src/entities/user.rs:23",
                                    "adrastos_core::entities::user",
                                    ::tracing::Level::ERROR,
                                    Some("crates/core/src/entities/user.rs"),
                                    Some(23u32),
                                    Some("adrastos_core::entities::user"),
                                    ::tracing_core::field::FieldSet::new(
                                        &["error"],
                                        ::tracing_core::callsite::Identifier(&CALLSITE),
                                    ),
                                    ::tracing::metadata::Kind::EVENT,
                                )
                            };
                            ::tracing::callsite::DefaultCallsite::new(&META)
                        };
                        let enabled = ::tracing::Level::ERROR
                            <= ::tracing::level_filters::STATIC_MAX_LEVEL
                            && ::tracing::Level::ERROR
                                <= ::tracing::level_filters::LevelFilter::current()
                            && {
                                let interest = CALLSITE.interest();
                                !interest.is_never()
                                    && ::tracing::__macro_support::__is_enabled(
                                        CALLSITE.metadata(),
                                        interest,
                                    )
                            };
                        if enabled {
                            (|value_set: ::tracing::field::ValueSet| {
                                let meta = CALLSITE.metadata();
                                ::tracing::Event::dispatch(meta, &value_set);
                                if (match ::tracing::Level::ERROR {
                                    ::tracing::Level::ERROR => ::tracing::log::Level::Error,
                                    ::tracing::Level::WARN => ::tracing::log::Level::Warn,
                                    ::tracing::Level::INFO => ::tracing::log::Level::Info,
                                    ::tracing::Level::DEBUG => ::tracing::log::Level::Debug,
                                    _ => ::tracing::log::Level::Trace,
                                }) <= ::tracing::log::STATIC_MAX_LEVEL
                                {
                                    if !::tracing::dispatcher::has_been_set() {
                                        {
                                            use ::tracing::log;
                                            let level = match ::tracing::Level::ERROR {
                                                ::tracing::Level::ERROR => ::tracing::log::Level::Error,
                                                ::tracing::Level::WARN => ::tracing::log::Level::Warn,
                                                ::tracing::Level::INFO => ::tracing::log::Level::Info,
                                                ::tracing::Level::DEBUG => ::tracing::log::Level::Debug,
                                                _ => ::tracing::log::Level::Trace,
                                            };
                                            if level <= log::max_level() {
                                                let meta = CALLSITE.metadata();
                                                let log_meta = log::Metadata::builder()
                                                    .level(level)
                                                    .target(meta.target())
                                                    .build();
                                                let logger = log::logger();
                                                if logger.enabled(&log_meta) {
                                                    ::tracing::__macro_support::__tracing_log(
                                                        meta,
                                                        logger,
                                                        log_meta,
                                                        &value_set,
                                                    )
                                                }
                                            }
                                        }
                                    } else {
                                        {}
                                    }
                                } else {
                                    {}
                                };
                            })({
                                #[allow(unused_imports)]
                                use ::tracing::field::{debug, display, Value};
                                let mut iter = CALLSITE.metadata().fields().iter();
                                CALLSITE
                                    .metadata()
                                    .fields()
                                    .value_set(
                                        &[
                                            (
                                                &iter.next().expect("FieldSet corrupted (this is a bug)"),
                                                Some(&debug(&e) as &dyn Value),
                                            ),
                                        ],
                                    )
                            });
                        } else {
                            if (match ::tracing::Level::ERROR {
                                ::tracing::Level::ERROR => ::tracing::log::Level::Error,
                                ::tracing::Level::WARN => ::tracing::log::Level::Warn,
                                ::tracing::Level::INFO => ::tracing::log::Level::Info,
                                ::tracing::Level::DEBUG => ::tracing::log::Level::Debug,
                                _ => ::tracing::log::Level::Trace,
                            }) <= ::tracing::log::STATIC_MAX_LEVEL
                            {
                                if !::tracing::dispatcher::has_been_set() {
                                    {
                                        use ::tracing::log;
                                        let level = match ::tracing::Level::ERROR {
                                            ::tracing::Level::ERROR => ::tracing::log::Level::Error,
                                            ::tracing::Level::WARN => ::tracing::log::Level::Warn,
                                            ::tracing::Level::INFO => ::tracing::log::Level::Info,
                                            ::tracing::Level::DEBUG => ::tracing::log::Level::Debug,
                                            _ => ::tracing::log::Level::Trace,
                                        };
                                        if level <= log::max_level() {
                                            let meta = CALLSITE.metadata();
                                            let log_meta = log::Metadata::builder()
                                                .level(level)
                                                .target(meta.target())
                                                .build();
                                            let logger = log::logger();
                                            if logger.enabled(&log_meta) {
                                                ::tracing::__macro_support::__tracing_log(
                                                    meta,
                                                    logger,
                                                    log_meta,
                                                    &{
                                                        #[allow(unused_imports)]
                                                        use ::tracing::field::{debug, display, Value};
                                                        let mut iter = CALLSITE.metadata().fields().iter();
                                                        CALLSITE
                                                            .metadata()
                                                            .fields()
                                                            .value_set(
                                                                &[
                                                                    (
                                                                        &iter.next().expect("FieldSet corrupted (this is a bug)"),
                                                                        Some(&debug(&e) as &dyn Value),
                                                                    ),
                                                                ],
                                                            )
                                                    },
                                                )
                                            }
                                        }
                                    }
                                } else {
                                    {}
                                }
                            } else {
                                {}
                            };
                        }
                    };
                    crate::error::Error::InternalServerError({
                        let res = ::alloc::fmt::format(
                            format_args!(
                                "Failed to delete {0}",
                                Self::error_identifier(),
                            ),
                        );
                        res
                    })
                })?;
            Ok(())
        }
    }
    pub enum UserIden {
        Table,
        Id,
        FirstName,
        LastName,
        Email,
        Username,
        Password,
        Verified,
        Banned,
        MfaSecret,
        MfaBackupCodes,
        CreatedAt,
        UpdatedAt,
        Connections,
        RefreshTokenTrees,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UserIden {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    UserIden::Table => "Table",
                    UserIden::Id => "Id",
                    UserIden::FirstName => "FirstName",
                    UserIden::LastName => "LastName",
                    UserIden::Email => "Email",
                    UserIden::Username => "Username",
                    UserIden::Password => "Password",
                    UserIden::Verified => "Verified",
                    UserIden::Banned => "Banned",
                    UserIden::MfaSecret => "MfaSecret",
                    UserIden::MfaBackupCodes => "MfaBackupCodes",
                    UserIden::CreatedAt => "CreatedAt",
                    UserIden::UpdatedAt => "UpdatedAt",
                    UserIden::Connections => "Connections",
                    UserIden::RefreshTokenTrees => "RefreshTokenTrees",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for UserIden {
        #[inline]
        fn clone(&self) -> UserIden {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UserIden {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for UserIden {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for UserIden {
        #[inline]
        fn eq(&self, other: &UserIden) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for UserIden {}
    #[automatically_derived]
    impl ::core::cmp::Eq for UserIden {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    #[automatically_derived]
    impl ::core::hash::Hash for UserIden {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            ::core::hash::Hash::hash(&__self_tag, state)
        }
    }
    impl sea_query::Iden for UserIden {
        fn unquoted(&self, s: &mut dyn sea_query::Write) {
            s.write_fmt(
                    format_args!(
                        "{0}",
                        match self {
                            UserIden::Table => "user",
                            UserIden::Id => "id",
                            UserIden::FirstName => "first_name",
                            UserIden::LastName => "last_name",
                            UserIden::Email => "email",
                            UserIden::Username => "username",
                            UserIden::Password => "password",
                            UserIden::Verified => "verified",
                            UserIden::Banned => "banned",
                            UserIden::MfaSecret => "mfa_secret",
                            UserIden::MfaBackupCodes => "mfa_backup_codes",
                            UserIden::CreatedAt => "created_at",
                            UserIden::UpdatedAt => "updated_at",
                            UserIden::Connections => "connections",
                            UserIden::RefreshTokenTrees => "refresh_token_trees",
                        },
                    ),
                )
                .unwrap();
        }
    }
    pub struct UpdateUser {
        #[validate(length(max = 50))]
        pub first_name: Option<String>,
        #[validate(length(max = 50))]
        pub last_name: Option<String>,
        #[validate(email)]
        pub email: Option<String>,
        #[validate(length(min = 5, max = 64))]
        pub username: Option<String>,
        #[validate(length(min = 8, max = 64))]
        pub password: Option<String>,
        pub verified: Option<bool>,
        pub banned: Option<bool>,
        pub mfa_secret: Option<Option<String>>,
        pub mfa_backup_codes: Option<Option<Vec<String>>>,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UpdateUser {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "first_name",
                "last_name",
                "email",
                "username",
                "password",
                "verified",
                "banned",
                "mfa_secret",
                "mfa_backup_codes",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.first_name,
                &self.last_name,
                &self.email,
                &self.username,
                &self.password,
                &self.verified,
                &self.banned,
                &self.mfa_secret,
                &&self.mfa_backup_codes,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "UpdateUser",
                names,
                values,
            )
        }
    }
    impl ::validator::Validate for UpdateUser {
        fn validate(&self) -> ::std::result::Result<(), ::validator::ValidationErrors> {
            use ::validator::ValidateArgs;
            self.validate_args(())
        }
    }
    #[allow(clippy::all)]
    #[allow(single_use_lifetimes)]
    impl<'v_a> ::validator::ValidateArgs<'v_a> for UpdateUser {
        type Args = ();
        #[allow(unused_mut)]
        #[allow(unused_variable)]
        fn validate_args(
            &self,
            args: Self::Args,
        ) -> ::std::result::Result<(), ::validator::ValidationErrors> {
            let mut errors = ::validator::ValidationErrors::new();
            if let Some(ref first_name) = self.first_name {
                if !::validator::validate_length(
                    first_name,
                    ::std::option::Option::None,
                    ::std::option::Option::Some(50u64 as u64),
                    ::std::option::Option::None,
                ) {
                    let mut err = ::validator::ValidationError::new("length");
                    err.add_param(::std::borrow::Cow::from("max"), &50u64);
                    err.add_param(::std::borrow::Cow::from("value"), &first_name);
                    errors.add("first_name", err);
                }
            }
            if let Some(ref last_name) = self.last_name {
                if !::validator::validate_length(
                    last_name,
                    ::std::option::Option::None,
                    ::std::option::Option::Some(50u64 as u64),
                    ::std::option::Option::None,
                ) {
                    let mut err = ::validator::ValidationError::new("length");
                    err.add_param(::std::borrow::Cow::from("max"), &50u64);
                    err.add_param(::std::borrow::Cow::from("value"), &last_name);
                    errors.add("last_name", err);
                }
            }
            if let Some(ref email) = self.email {
                if !::validator::validate_email(email) {
                    let mut err = ::validator::ValidationError::new("email");
                    err.add_param(::std::borrow::Cow::from("value"), &email);
                    errors.add("email", err);
                }
            }
            if let Some(ref username) = self.username {
                if !::validator::validate_length(
                    username,
                    ::std::option::Option::Some(5u64 as u64),
                    ::std::option::Option::Some(64u64 as u64),
                    ::std::option::Option::None,
                ) {
                    let mut err = ::validator::ValidationError::new("length");
                    err.add_param(::std::borrow::Cow::from("min"), &5u64);
                    err.add_param(::std::borrow::Cow::from("max"), &64u64);
                    err.add_param(::std::borrow::Cow::from("value"), &username);
                    errors.add("username", err);
                }
            }
            if let Some(ref password) = self.password {
                if !::validator::validate_length(
                    password,
                    ::std::option::Option::Some(8u64 as u64),
                    ::std::option::Option::Some(64u64 as u64),
                    ::std::option::Option::None,
                ) {
                    let mut err = ::validator::ValidationError::new("length");
                    err.add_param(::std::borrow::Cow::from("min"), &8u64);
                    err.add_param(::std::borrow::Cow::from("max"), &64u64);
                    err.add_param(::std::borrow::Cow::from("value"), &password);
                    errors.add("password", err);
                }
            }
            let mut result = if errors.is_empty() {
                ::std::result::Result::Ok(())
            } else {
                ::std::result::Result::Err(errors)
            };
            result
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for UpdateUser {
        #[inline]
        fn clone(&self) -> UpdateUser {
            UpdateUser {
                first_name: ::core::clone::Clone::clone(&self.first_name),
                last_name: ::core::clone::Clone::clone(&self.last_name),
                email: ::core::clone::Clone::clone(&self.email),
                username: ::core::clone::Clone::clone(&self.username),
                password: ::core::clone::Clone::clone(&self.password),
                verified: ::core::clone::Clone::clone(&self.verified),
                banned: ::core::clone::Clone::clone(&self.banned),
                mfa_secret: ::core::clone::Clone::clone(&self.mfa_secret),
                mfa_backup_codes: ::core::clone::Clone::clone(&self.mfa_backup_codes),
            }
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for UpdateUser {
        #[inline]
        fn default() -> UpdateUser {
            UpdateUser {
                first_name: ::core::default::Default::default(),
                last_name: ::core::default::Default::default(),
                email: ::core::default::Default::default(),
                username: ::core::default::Default::default(),
                password: ::core::default::Default::default(),
                verified: ::core::default::Default::default(),
                banned: ::core::default::Default::default(),
                mfa_secret: ::core::default::Default::default(),
                mfa_backup_codes: ::core::default::Default::default(),
            }
        }
    }
    impl User {
        pub async fn update(
            &self,
            db_pool: &deadpool_postgres::Pool,
            update: UpdateUser,
        ) -> Result<(), Error> {
            update
                .validate()
                .map_err(|e| Error::ValidationErrors {
                    errors: e,
                    message: "Invalid user update".into(),
                })?;
            let query = sea_query::Query::update()
                .table(Self::table())
                .values(
                    Update::create([
                        (UserIden::FirstName, update.first_name.into()),
                        (UserIden::LastName, update.last_name.into()),
                        (UserIden::Email, update.email.into()),
                        (UserIden::Username, update.username.into()),
                        (
                            UserIden::Password,
                            update
                                .password
                                .map(|v| auth::hash_password(v.as_str()).unwrap_or_log())
                                .into(),
                        ),
                        (UserIden::Verified, update.verified.into()),
                        (UserIden::Banned, update.banned.into()),
                        (UserIden::MfaSecret, update.mfa_secret.into()),
                        (UserIden::MfaBackupCodes, update.mfa_backup_codes.into()),
                        (UserIden::UpdatedAt, Some(Utc::now()).into()),
                    ]),
                )
                .and_where(Expr::col(UserIden::Id).eq(self.id.clone()))
                .to_string(PostgresQueryBuilder);
            db_pool
                .get()
                .await
                .unwrap_or_log()
                .execute(&query, &[])
                .await
                .map_err(|e| {
                    {
                        use ::tracing::__macro_support::Callsite as _;
                        static CALLSITE: ::tracing::callsite::DefaultCallsite = {
                            static META: ::tracing::Metadata<'static> = {
                                ::tracing_core::metadata::Metadata::new(
                                    "event crates/core/src/entities/user.rs:120",
                                    "adrastos_core::entities::user",
                                    ::tracing::Level::ERROR,
                                    Some("crates/core/src/entities/user.rs"),
                                    Some(120u32),
                                    Some("adrastos_core::entities::user"),
                                    ::tracing_core::field::FieldSet::new(
                                        &["error"],
                                        ::tracing_core::callsite::Identifier(&CALLSITE),
                                    ),
                                    ::tracing::metadata::Kind::EVENT,
                                )
                            };
                            ::tracing::callsite::DefaultCallsite::new(&META)
                        };
                        let enabled = ::tracing::Level::ERROR
                            <= ::tracing::level_filters::STATIC_MAX_LEVEL
                            && ::tracing::Level::ERROR
                                <= ::tracing::level_filters::LevelFilter::current()
                            && {
                                let interest = CALLSITE.interest();
                                !interest.is_never()
                                    && ::tracing::__macro_support::__is_enabled(
                                        CALLSITE.metadata(),
                                        interest,
                                    )
                            };
                        if enabled {
                            (|value_set: ::tracing::field::ValueSet| {
                                let meta = CALLSITE.metadata();
                                ::tracing::Event::dispatch(meta, &value_set);
                                if (match ::tracing::Level::ERROR {
                                    ::tracing::Level::ERROR => ::tracing::log::Level::Error,
                                    ::tracing::Level::WARN => ::tracing::log::Level::Warn,
                                    ::tracing::Level::INFO => ::tracing::log::Level::Info,
                                    ::tracing::Level::DEBUG => ::tracing::log::Level::Debug,
                                    _ => ::tracing::log::Level::Trace,
                                }) <= ::tracing::log::STATIC_MAX_LEVEL
                                {
                                    if !::tracing::dispatcher::has_been_set() {
                                        {
                                            use ::tracing::log;
                                            let level = match ::tracing::Level::ERROR {
                                                ::tracing::Level::ERROR => ::tracing::log::Level::Error,
                                                ::tracing::Level::WARN => ::tracing::log::Level::Warn,
                                                ::tracing::Level::INFO => ::tracing::log::Level::Info,
                                                ::tracing::Level::DEBUG => ::tracing::log::Level::Debug,
                                                _ => ::tracing::log::Level::Trace,
                                            };
                                            if level <= log::max_level() {
                                                let meta = CALLSITE.metadata();
                                                let log_meta = log::Metadata::builder()
                                                    .level(level)
                                                    .target(meta.target())
                                                    .build();
                                                let logger = log::logger();
                                                if logger.enabled(&log_meta) {
                                                    ::tracing::__macro_support::__tracing_log(
                                                        meta,
                                                        logger,
                                                        log_meta,
                                                        &value_set,
                                                    )
                                                }
                                            }
                                        }
                                    } else {
                                        {}
                                    }
                                } else {
                                    {}
                                };
                            })({
                                #[allow(unused_imports)]
                                use ::tracing::field::{debug, display, Value};
                                let mut iter = CALLSITE.metadata().fields().iter();
                                CALLSITE
                                    .metadata()
                                    .fields()
                                    .value_set(
                                        &[
                                            (
                                                &iter.next().expect("FieldSet corrupted (this is a bug)"),
                                                Some(&debug(&e) as &dyn Value),
                                            ),
                                        ],
                                    )
                            });
                        } else {
                            if (match ::tracing::Level::ERROR {
                                ::tracing::Level::ERROR => ::tracing::log::Level::Error,
                                ::tracing::Level::WARN => ::tracing::log::Level::Warn,
                                ::tracing::Level::INFO => ::tracing::log::Level::Info,
                                ::tracing::Level::DEBUG => ::tracing::log::Level::Debug,
                                _ => ::tracing::log::Level::Trace,
                            }) <= ::tracing::log::STATIC_MAX_LEVEL
                            {
                                if !::tracing::dispatcher::has_been_set() {
                                    {
                                        use ::tracing::log;
                                        let level = match ::tracing::Level::ERROR {
                                            ::tracing::Level::ERROR => ::tracing::log::Level::Error,
                                            ::tracing::Level::WARN => ::tracing::log::Level::Warn,
                                            ::tracing::Level::INFO => ::tracing::log::Level::Info,
                                            ::tracing::Level::DEBUG => ::tracing::log::Level::Debug,
                                            _ => ::tracing::log::Level::Trace,
                                        };
                                        if level <= log::max_level() {
                                            let meta = CALLSITE.metadata();
                                            let log_meta = log::Metadata::builder()
                                                .level(level)
                                                .target(meta.target())
                                                .build();
                                            let logger = log::logger();
                                            if logger.enabled(&log_meta) {
                                                ::tracing::__macro_support::__tracing_log(
                                                    meta,
                                                    logger,
                                                    log_meta,
                                                    &{
                                                        #[allow(unused_imports)]
                                                        use ::tracing::field::{debug, display, Value};
                                                        let mut iter = CALLSITE.metadata().fields().iter();
                                                        CALLSITE
                                                            .metadata()
                                                            .fields()
                                                            .value_set(
                                                                &[
                                                                    (
                                                                        &iter.next().expect("FieldSet corrupted (this is a bug)"),
                                                                        Some(&debug(&e) as &dyn Value),
                                                                    ),
                                                                ],
                                                            )
                                                    },
                                                )
                                            }
                                        }
                                    }
                                } else {
                                    {}
                                }
                            } else {
                                {}
                            };
                        }
                    };
                    Error::InternalServerError("Failed to update user".into())
                })?;
            Ok(())
        }
    }
}
