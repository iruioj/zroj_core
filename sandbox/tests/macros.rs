#[test]
#[cfg(all(unix))]
#[cfg_attr(not(unix), ignore = "not unix os")]
fn text_macros() {
    use sandbox::{
        sigton,
        unix::{Limitation, SingletonBuilder},
        Builder,
    };

    // sigton!{
    //     exec: "/usr/bin/sleep";
    //     cmd: "sleep" "2";
    // };
    let s = sigton! {
        exec: "/usr/bin/sleep";
        cmd: "sleep" "2";
        lim cpu_time: 1000 3000;
        lim real_time: 2000;
        lim real_memory: 256 * 1024 * 1024;
        lim virtual_memory: 256 * 1024 * 1024 1024 * 1024 * 1024;
        lim stack: 256 * 1024 * 1024 1024 * 1024 * 1024;
        lim output: 256 * 1024 * 1024 1024 * 1024 * 1024;
        lim fileno: 10 10;
    };
    let s2 = SingletonBuilder::new("/usr/bin/sleep")
        .push_arg("sleep")
        .push_arg("2")
        .set_limits(|_| Limitation {
            real_time: Some(2000),
            cpu_time: Some((1000, 3000)),
            virtual_memory: Some((256 * 1024 * 1024, 1024 * 1024 * 102)),
            real_memory: Some(256 * 1024 * 1024),
            stack_memory: Some((256 * 1024 * 1024, 1024 * 1024 * 102)),
            output_memory: Some((256 * 1024 * 1024, 1024 * 1024 * 102)),
            fileno: Some((10, 10)),
        })
        .build().unwrap();
    dbg!(s, s2);
}
