use sandbox_macro::mem;
use sandbox_macro::time;
use sandbox::{Memory, Elapse};

#[test]
#[cfg(all(unix))]
#[cfg_attr(not(target_os = "linux"), ignore = "not linux")]
fn test_macro() {
    let t = time!(100ms);
    let t2 = time!(100s);
    let m = mem!(100m);
    let m2 = mem!(100g);
    dbg!(t, t2, m, m2);
}
