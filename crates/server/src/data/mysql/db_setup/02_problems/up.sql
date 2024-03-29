-- Your SQL goes here
CREATE TABLE problems (
    id integer unsigned NOT NULL AUTO_INCREMENT,
    title tinytext NOT NULL,
    meta mediumtext NOT NULL,
    PRIMARY KEY (id)
);
CREATE TABLE problem_statements (
    id integer unsigned NOT NULL AUTO_INCREMENT, -- useless
    pid integer unsigned NOT NULL,
    content mediumtext NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (pid) REFERENCES problems(id)
);