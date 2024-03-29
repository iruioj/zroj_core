-- Your SQL goes here
CREATE TABLE submission_metas (
    id integer unsigned NOT NULL AUTO_INCREMENT, -- submission's id
    pid integer unsigned NOT NULL,
    uid integer unsigned NOT NULL,
    submit_time bigint NOT NULL,

    judge_time bigint,
    lang tinytext,
    status tinytext,
    time bigint unsigned,
    memory bigint unsigned,
    
    PRIMARY KEY (id),
    FOREIGN KEY(pid) REFERENCES problems(id),
    FOREIGN KEY(uid) REFERENCES users(id)
);
CREATE TABLE submission_details (
    id integer unsigned NOT NULL AUTO_INCREMENT, -- useless
    sid integer unsigned NOT NULL,
    raw mediumtext NOT NULL,
    report mediumtext,

    PRIMARY KEY (id),
    FOREIGN KEY(sid) REFERENCES submission_metas(id)
);