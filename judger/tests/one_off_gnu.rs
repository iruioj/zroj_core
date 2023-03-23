

#[cfg(test)]
mod one_off {
    use judger::{OneOff, GnuCpp17O2, Error, Status};

    #[test]
    fn test_gnu_cpp() -> Result<(), Error> {
        let one = OneOff::new("assets/a_plus_b.cpp".into(), GnuCpp17O2{});
        let res = one.exec()?;
        if let Status::Accepted = res.status {
            eprintln!("res = {:?}", res);
        } else {
            panic!("compile failed")
        }
        Ok(())
    }
}