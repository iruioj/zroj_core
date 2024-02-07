use server::manager::JobRunner;

fn main() {
    let runner = JobRunner::new();
    println!("create");
    runner.add_job(|| println!("hello world")).unwrap();
    runner.terminate_join().unwrap(); // `drop(runner)` is ok but not recommended
}
