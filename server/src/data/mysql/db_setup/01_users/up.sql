-- Your SQL goes here
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