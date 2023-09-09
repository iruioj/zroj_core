CREATE TABLE users (
    id integer unsigned NOT NULL AUTO_INCREMENT,
    username tinytext NOT NULL,
    password_hash tinytext NOT NULL,
    name tinytext NOT NULL,
    email tinytext NOT NULL,
    motto tinytext NOT NULL,
    register_time bigint NOT NULL,
    gender tinytext NOT NULL,
    PRIMARY KEY (id)
);
CREATE TABLE problem_statements (
    pid integer unsigned NOT NULL AUTO_INCREMENT,
    title tinytext NOT NULL,
    content mediumtext NOT NULL,
    meta mediumtext NOT NULL,
    PRIMARY KEY (pid)
);