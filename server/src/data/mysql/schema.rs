
// 必须保证和 User 的字段顺序相同， 不然 query 会出问题
diesel::table! {
    users (id) {
        /// id should be auto increment
        id -> Unsigned<Integer>,
        username -> Text,
        password_hash -> Text,
        name -> Text,
        email -> Text,
        motto -> Text,
        register_time -> BigInt,
        gender -> Text,
    }
}

diesel::table! {
    /// 存储题面以及相关元信息，用于批量查询（一个大缓存）；源数据仍然与 ojdata 存储在一起
    problem_statements (pid) {
        pid -> Unsigned<Integer>,
        title -> Text,
        content -> Text,
        meta -> Text,
    }
}