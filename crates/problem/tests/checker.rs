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

#[test]
fn test_cabi_cpp() -> anyhow::Result<()> {
    let mut checker = Checker::CABI {
        source: SourceFile::from_str(
            r#"
#include<iostream>
#include "checker_c_abi.h"

extern "C" {
    // it is not required to include the checker_c_abi.h header (but is highly recommended).
    // make sure to have the right signature! otherwises the behavior is undefined
    float check() {
        std::cout << "run checker!" << std::endl;
        return 1.0;
    }
}"#,
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
    dbg!(r);

    Ok(())
}

#[test]
fn test_cabi_rust() -> anyhow::Result<()> {
    let mut checker = Checker::CABI {
        source: SourceFile::from_str(
            r#"
#[no_mangle]
pub extern "C" fn check() -> f32 {
    let v = vec![1,2,3];
    println!("run rust checker!, v = {v:?}");
    1.0
}"#,
            judger::FileType::Rust,
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
    dbg!(r);

    Ok(())
}
