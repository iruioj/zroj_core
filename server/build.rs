fn main() {
    println!("cargo:rerun-if-changed=./src/data/mysql/db_setup");
}
