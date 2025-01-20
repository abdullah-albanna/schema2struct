# schema2struct: JSON Schema to Rust Struct Generator

A powerful procedural macro for generating type-safe Rust structs from JSON Schema definitions with compile-time validation and Serde integration.


## Features

-  Automatic struct generation from JSON Schema

-  Compile-time type checking

-  Support for complex nested schemas

-  Constraint validation

-  Zero-cost abstraction


## Installation


Add the following to your `Cargo.toml`:


```toml

[dependencies]

schema2struct = "0.1"

serde = { version = "1.0", features = ["derive"] }

serde_json = "1.0"
```

## Quick Start

### Basic Usage 

```rust
use schema2struct::schema2struct;

schema2struct! {
    struct: User,
    type: object,
    properties: {
        "name": { type: string },
        "age": { type: number, minimum: 0 }
    },
    required: ["name", "age"]
}
```

#### Output

```rust

pub static USER_JSON_VALUE: ::std::sync::LazyLock<::serde_json::Value> = ::std::sync::LazyLock::new(||
{
    ::serde_json::from_str(
            "{\"type\":\"object\",\"properties\":{\"name\":{\"type\":\"string\"},\"age\":{\"type\":\"number\",\"minimum\":0}},\"required\":[\"name\",\"age\"]}",
        )
        .expect("Couldn't convert the text into valid json")
});

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(alias = "age")]
    pub age: f64,
    #[serde(alias = "name")]
    pub name: String,
}
```
**more complex usages can be found in the examples folder**

## License
Distributed under the MIT License. See LICENSE for more information.
