# Api Error Derive

This crate provides a simple derive macro for creating own API errors.

# Examples
```rust
use api_error_derive::{ApiError, ApiErrorData};
use http::StatusCode;
use thiserror::Error;

#[derive(ApiError, Debug, Error)]
enum MyError {
    #[error("example message ({0})")]
    WithoutAnyAttributes(i32),

    #[error("example message 2")]
    #[pass]
    PassAttribute,

    // #[status_code] activates the #[pass] attribute by default
    #[error("example message 3")]
    #[status_code(BAD_REQUEST)]
    StatusCodeAttribute,

    #[error("example message 4")]
    #[custom("CustomErrorMessage")]
    CustomAttribute,

    #[error("example message 5")]
    #[status_code(UNAUTHORIZED)]
    #[custom("CustomErrorMessage2")]
    StatusCodeAndCustomAttributes,
}

let err: ApiErrorData = MyError::WithoutAnyAttributes(123).into();
assert_eq!(err.status_code, StatusCode::INTERNAL_SERVER_ERROR);
assert_eq!(err.description, "example message (123)");
assert_eq!(err.client_description, "InternalServerError");

let err: ApiErrorData = MyError::PassAttribute.into();
assert_eq!(err.status_code, StatusCode::INTERNAL_SERVER_ERROR);
assert_eq!(err.description, "example message 2");
assert_eq!(err.client_description, "PassAttribute");

let err: ApiErrorData = MyError::StatusCodeAttribute.into();
assert_eq!(err.status_code, StatusCode::BAD_REQUEST);
assert_eq!(err.description, "example message 3");
assert_eq!(err.client_description, "StatusCodeAttribute");

let err: ApiErrorData = MyError::CustomAttribute.into();
assert_eq!(err.status_code, StatusCode::INTERNAL_SERVER_ERROR);
assert_eq!(err.description, "example message 4");
assert_eq!(err.client_description, "CustomErrorMessage");

let err: ApiErrorData = MyError::StatusCodeAndCustomAttributes.into();
assert_eq!(err.status_code, StatusCode::UNAUTHORIZED);
assert_eq!(err.description, "example message 5");
assert_eq!(err.client_description, "CustomErrorMessage2");
```

The `#[pass]` attribute is excess with `#[status_code(...)]` or `#[custom(...)]` macros.
```rust
# use api_error_derive::ApiError;
# use thiserror::Error;
#[derive(ApiError, Debug, Error)]
enum Error {
    #[error("example")]
    #[status_code(BAD_REQUEST)] // or #[custom(...)]
    #[pass] // Compile error!
    A,
}
```

# Details
- An API error enum should implement the `core::fmt::Display` trait. Recommend using the [thiserror] crate for this. 
- `#[status_code(value)]` accepts only [http::StatusCode associated constants].

# Features
The `axum` feature also provides the [IntoResponse] trait.

[thiserror]: https://docs.rs/thiserror/latest/thiserror/
[http::StatusCode associated constants]: http::StatusCode#impl-StatusCode-1
[IntoResponse]: https://docs.rs/axum/latest/axum/response/trait.IntoResponse.html