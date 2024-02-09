-- Your SQL goes here
CREATE TABLE problems (
    id integer unsigned NOT NULL AUTO_INCREMENT,
    title tinytext NOT NULL,
    meta mediumtext NOT NULL,
    PRIMARY KEY (id)
);