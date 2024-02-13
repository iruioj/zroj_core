use judger::{DefaultJudger, Judger, SourceFile, StoreFile};
use problem::Checker;
use store::Handle;

#[test]
fn test_testlib() -> anyhow::Result<()> {
    let mut checker = Checker::TestlibChecker {
        testlib_header: StoreFile::from_str(
            include_str!("assets/testlib.txt"),
            judger::FileType::GnuCpp14O2,
        ),
        checker: SourceFile::from_str(
            include_str!("assets/acmp.txt"),
            judger::FileType::GnuCpp14O2,
        ),
    };

    let wd = tempfile::tempdir().unwrap();

    let mut judger = DefaultJudger::new(Handle::new(wd.path()), None);
    judger.runtime_log("");
    let input = <DefaultJudger as Judger<String>>::copy_store_file(
        &judger,
        &mut StoreFile::from_str("0", judger::FileType::Plain),
        "input",
    )?;
    let output = <DefaultJudger as Judger<String>>::copy_store_file(
        &judger,
        &mut StoreFile::from_str("0", judger::FileType::Plain),
        "output",
    )?;
    let answer = <DefaultJudger as Judger<String>>::copy_store_file(
        &judger,
        &mut StoreFile::from_str("0.0001", judger::FileType::Plain),
        "answer",
    )?;

    let r = checker.check::<String>(&judger, &input, &output, &answer)?;
    assert!(r.0.abs() < 1.0e-5);
    dbg!(r);

    Ok(())
}
