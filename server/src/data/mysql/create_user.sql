-- login in 'root' (you can sudo and then `mysql -u root`)
DROP USER IF EXISTS 'test'@'localhost';
CREATE USER 'test'@'localhost' IDENTIFIED BY 'test';
