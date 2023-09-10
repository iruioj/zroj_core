-- Your SQL goes here
CREATE TABLE problem_statements (
    pid integer unsigned NOT NULL AUTO_INCREMENT,
    title tinytext NOT NULL,
    content mediumtext NOT NULL,
    meta mediumtext NOT NULL,
    PRIMARY KEY (pid)
);