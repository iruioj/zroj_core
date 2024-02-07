// @generated automatically by Diesel CLI.

diesel::table! {
    problem_statements (id) {
        id -> Unsigned<Integer>,
        pid -> Unsigned<Integer>,
        content -> Mediumtext,
    }
}

diesel::table! {
    problems (id) {
        id -> Unsigned<Integer>,
        title -> Tinytext,
        meta -> Mediumtext,
    }
}

diesel::table! {
    submission_details (id) {
        id -> Unsigned<Integer>,
        sid -> Unsigned<Integer>,
        raw -> Mediumtext,
        report -> Nullable<Mediumtext>,
    }
}

diesel::table! {
    submission_metas (id) {
        id -> Unsigned<Integer>,
        pid -> Unsigned<Integer>,
        uid -> Unsigned<Integer>,
        submit_time -> Bigint,
        judge_time -> Nullable<Bigint>,
        lang -> Nullable<Tinytext>,
        status -> Nullable<Tinytext>,
        time -> Nullable<Unsigned<Bigint>>,
        memory -> Nullable<Unsigned<Bigint>>,
    }
}

diesel::table! {
    users (id) {
        id -> Unsigned<Integer>,
        username -> Tinytext,
        password_hash -> Tinytext,
        name -> Tinytext,
        email -> Tinytext,
        motto -> Tinytext,
        register_time -> Bigint,
        gender -> Tinytext,
    }
}

diesel::joinable!(problem_statements -> problems (pid));
diesel::joinable!(submission_details -> submission_metas (sid));
diesel::joinable!(submission_metas -> problems (pid));
diesel::joinable!(submission_metas -> users (uid));

diesel::allow_tables_to_appear_in_same_query!(
    problem_statements,
    problems,
    submission_details,
    submission_metas,
    users,
);
