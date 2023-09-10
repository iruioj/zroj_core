-- Your SQL goes here
CREATE TABLE submission_details (
    id integer unsigned NOT NULL AUTO_INCREMENT,
    sid integer unsigned NOT NULL,
    raw mediumtext NOT NULL,
    report mediumtext,

    PRIMARY KEY (id),
    FOREIGN KEY(sid) REFERENCES submission_metas(id)
);