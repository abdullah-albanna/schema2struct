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

fn main() {}
