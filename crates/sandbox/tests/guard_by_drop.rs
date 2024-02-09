use std::process::exit;

struct TestGuard(i32);

impl Drop for TestGuard {
    fn drop(&mut self) {
        println!("drop guard {}", self.0);
    }
}

// expected output:
//
//   do something
//   drop guard 2
//   done.
//   drop guard 1
//
fn inner_fn() {
    let _g1 = TestGuard(1);
    {
        let _g2 = TestGuard(2);
        println!("do something");
    }
    println!("done.");
}

#[test]
// g0 won't be dropped before exit, which is actually nothing, since the process has exists.
fn test_guard() {
    let _g0 = TestGuard(0);
    inner_fn();
    exit(0);
}
