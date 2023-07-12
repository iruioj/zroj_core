use serde_ts_typing::*;

#[test]
fn test_type_expr() {
    let ty = TypeExpr::Record(
        [
            ("name".into(), TypeExpr::String),
            ("flag".into(), TypeExpr::Boolean),
            (
                "children".into(),
                TypeExpr::Array(Box::new(TypeExpr::Tuple(
                    [
                        TypeExpr::Union(
                            [
                                TypeExpr::Value(Value::String("root".into())),
                                TypeExpr::Value(Value::String("leaf".into())),
                                TypeExpr::Value(Value::Null),
                            ]
                            .into_iter()
                            .collect(),
                        ),
                        TypeExpr::Number,
                    ]
                    .into_iter()
                    .collect(),
                ))),
            ),
        ]
        .into_iter()
        .collect(),
    );
    assert_eq!(
        ty.to_string(),
        "{children:[(null|\"leaf\"|\"root\"),number][];flag:boolean;name:string;}"
    );
}
