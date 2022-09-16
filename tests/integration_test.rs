use anyhow::Result;
use rust_solv::{repo, solve};
use std::fs;

#[test]
fn test_dependency_unsatisfied() -> Result<()> {
    let xml = fs::read_to_string("/root/rust-solv/tests/dependency-unsatisfied.xml")?;
    let repo = repo::Repo::from_str(&xml)?;
    match solve::check_package_satisfiability_in_repo(&repo, &"A".to_string()) {
        Ok(solve::ReturnValue::Satisfied) => println!(
            "Congratulations! Package {}'s dependencies can be satisfied in the repo. :)",
            "A"
        ),
        Ok(solve::ReturnValue::Unsatisfied) => println!(
            "Sorry, package {}'s dependencies can not be satisfied in the repo. :(",
            "A"
        ),
        Ok(solve::ReturnValue::VersionConflict) => println!(
            "Sorry, package {}'s dependencies can not be satisfied in the repo. (version conflict) :(",
            "A"
        ),
        Ok(solve::ReturnValue::PackageNotFound) => {
            println!("Error: package {} not found in the repo. :(", "A")
        }
        Err(_) => println!("Error: something wrong happened while solving. :("),
    }
    Ok(())
}

#[test]
fn test_version_unsatisfied() -> Result<()> {
    let xml = fs::read_to_string("/root/rust-solv/tests/version-unsatisfied.xml")?;
    let repo = repo::Repo::from_str(&xml)?;
    match solve::check_package_satisfiability_in_repo(&repo, &"A".to_string()) {
        Ok(solve::ReturnValue::Satisfied) => println!(
            "Congratulations! Package {}'s dependencies can be satisfied in the repo. :)",
            "A"
        ),
        Ok(solve::ReturnValue::Unsatisfied) => println!(
            "Sorry, package {}'s dependencies can not be satisfied in the repo. :(",
            "A"
        ),
        Ok(solve::ReturnValue::VersionConflict) => println!(
            "Sorry, package {}'s dependencies can not be satisfied in the repo. (version conflict) :(",
            "A"
        ),
        Ok(solve::ReturnValue::PackageNotFound) => {
            println!("Error: package {} not found in the repo. :(", "A")
        }
        Err(_) => println!("Error: something wrong happened while solving. :("),
    }
    Ok(())
}

#[test]
fn test_satisfied() -> Result<()> {
    let xml = fs::read_to_string("/root/rust-solv/tests/satisfied.xml")?;
    let repo = repo::Repo::from_str(&xml)?;
    match solve::check_package_satisfiability_in_repo(&repo, &"A".to_string()) {
        Ok(solve::ReturnValue::Satisfied) => println!(
            "Congratulations! Package {}'s dependencies can be satisfied in the repo. :)",
            "A"
        ),
        Ok(solve::ReturnValue::Unsatisfied) => println!(
            "Sorry, package {}'s dependencies can not be satisfied in the repo. :(",
            "A"
        ),
        Ok(solve::ReturnValue::VersionConflict) => println!(
            "Sorry, package {}'s dependencies can not be satisfied in the repo. (version conflict) :(",
            "A"
        ),
        Ok(solve::ReturnValue::PackageNotFound) => {
            println!("Error: package {} not found in the repo. :(", "A")
        }
        Err(_) => println!("Error: something wrong happened while solving. :("),
    }
    Ok(())
}
