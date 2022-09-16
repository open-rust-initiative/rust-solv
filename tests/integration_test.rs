use anyhow::Result;
use rust_solv::{repo, solve};
use std::fs;

#[test]
fn test_dependency_unsatisfied() -> Result<()> {
    let xml = fs::read_to_string("/root/rust-solv/tests/dependency-unsatisfied.xml")?;
    let repo = repo::Repo::from_str(&xml)?;
    match solve::check_package_satisfiability_in_repo(&repo, &"A".to_string()) {
        Ok(true) => println!(
            "Congratulations! Package {}'s dependencies can be satisfied in the repo. :)",
            "A"
        ),
        _ => println!(
            "Sorry, package {}'s dependencies can not be satisfied in the repo. :(",
            "A"
        ),
    }

    Ok(())
}

#[test]
fn test_version_unsatisfied() -> Result<()> {
    let xml = fs::read_to_string("/root/rust-solv/tests/version-unsatisfied.xml")?;
    let repo = repo::Repo::from_str(&xml)?;
    match solve::check_package_satisfiability_in_repo(&repo, &"A".to_string()) {
        Ok(true) => println!(
            "Congratulations! Package {}'s dependencies can be satisfied in the repo. :)",
            "A"
        ),
        _ => println!(
            "Sorry, package {}'s dependencies can not be satisfied in the repo. :(",
            "A"
        ),
    }

    Ok(())
}

#[test]
fn test_satisfied() -> Result<()> {
    let xml = fs::read_to_string("/root/rust-solv/tests/satisfied.xml")?;
    let repo = repo::Repo::from_str(&xml)?;
    match solve::check_package_satisfiability_in_repo(&repo, &"A".to_string()) {
        Ok(true) => println!(
            "Congratulations! Package {}'s dependencies can be satisfied in the repo. :)",
            "A"
        ),
        _ => println!(
            "Sorry, package {}'s dependencies can not be satisfied in the repo. :(",
            "A"
        ),
    }

    Ok(())
}