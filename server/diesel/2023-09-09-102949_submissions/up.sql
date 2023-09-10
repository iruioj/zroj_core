-- Your SQL goes here
CREATE TABLE submissions (
    id integer unsigned NOT NULL AUTO_INCREMENT,
    pid integer unsigned NOT NULL,
    uid integer unsigned NOT NULL,
    submit_time bigint NOT NULL,
    raw mediumtext NOT NULL,

    judge_time bigint,
    lang tinytext,
    status tinytext,
    time bigint unsigned,
    memory bigint unsigned,
    report mediumtext,
    PRIMARY KEY (id),
    FOREIGN KEY(pid) REFERENCES problem_statements(pid),
    FOREIGN KEY(uid) REFERENCES users(id)
);