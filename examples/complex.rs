use schema2struct::schema2struct;

schema2struct! {
    type: object,
    struct: Hello,
    enum: ["hello", 12, true],
    default: "hello",
    required: ["info", "anything", "sdc"],
    properties: {
        "sdc": {
            type: object,
            struct: key
        },
        "info": {
            type: number,
            minimum: 12,
            maximum: 90
        },

        "anything": {
            type: string,
            min_length: 52,
            max_length: 32,
            pattern: r#"a-zA-Z"#,
            format: email,

        },
    },
}

fn main() {
    let api_response_from_the_scehma = r#"
        {
            "info": 23,
            "anything": "hello",
            "sdc": {}
        }
        "#;

    let binded: Hello = serde_json::from_str(api_response_from_the_scehma).unwrap();

    println!("{:#?}", binded);
}
