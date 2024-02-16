-- Your SQL goes here
CREATE TABLE contests (
    id integer unsigned NOT NULL AUTO_INCREMENT,
    title tinytext NOT NULL,
    start_time bigint NOT NULL,
    end_time bigint NOT NULL,
    duration bigint unsigned NOT NULL,
    PRIMARY KEY (id)
);
CREATE TABLE contest_problems (
    pid integer unsigned NOT NULL,
    cid integer unsigned NOT NULL,

    PRIMARY KEY (pid, cid),
    -- FOREIGN KEY ( UserId ) REFERENCES [User] ( Id ) ON UPDATE  NO ACTION  ON DELETE  CASCADE
    FOREIGN KEY(pid) REFERENCES problems(id),
    FOREIGN KEY(cid) REFERENCES contests(id)
);
CREATE TABLE contest_registrants (
    cid integer unsigned NOT NULL,
    uid integer unsigned NOT NULL,
    register_time bigint NOT NULL, 

    PRIMARY KEY (cid, uid),
    FOREIGN KEY(uid) REFERENCES users(id),
    FOREIGN KEY(cid) REFERENCES contests(id)
);
CREATE TABLE contest_submissions (
    cid integer unsigned NOT NULL,
    sid integer unsigned NOT NULL,

    PRIMARY KEY (cid, sid),
    FOREIGN KEY(sid) REFERENCES submission_metas(id),
    FOREIGN KEY(cid) REFERENCES contests(id)
)