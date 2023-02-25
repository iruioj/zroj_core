use sandbox::sigton;

#[test]
#[cfg(all(unix))]
fn text_marcro() {
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
    dbg!(s);
}
