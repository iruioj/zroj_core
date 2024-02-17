use problem::{prelude::Traditional, StandardProblem};
use server::manager::ProblemJudger;
use store::Handle;

fn main() {
    let dir = tempfile::TempDir::new().unwrap();
    let dir_handle = Handle::new(dir.path());
    let problem_judger = ProblemJudger::new(dir_handle).unwrap();

    let StandardProblem::Traditional(ojdata) = problem::sample::a_plus_b_data();
    let subm = problem::sample::a_plus_b_std();

    problem_judger
        .add_test::<Traditional>(0, ojdata, subm)
        .unwrap();
    println!("test added");

    let (_, rep) = problem_judger.reciver().recv().unwrap();

    dbg!(rep);

    drop(problem_judger);
    drop(dir)
}
