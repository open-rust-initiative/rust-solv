use crate::repo::{IdT, Repo};
use anyhow::{anyhow, Result};
use std::collections::{HashSet, VecDeque};
use varisat::{solver::Solver, CnfFormula, ExtendFormula, Lit};

fn get_formula_by_package_id(repo: &Repo, package_id: IdT) -> Result<CnfFormula> {
    let mut q = VecDeque::new();
    let mut formula = CnfFormula::new();
    let mut appeared = HashSet::new();
    q.push_back(package_id);
    appeared.insert(package_id);
    while let Some(package_id) = q.pop_front() {
        if let Some(requires) = repo.get_package_requires_by_id(package_id) {
            for entry in requires {
                if let Some(providers) = repo.get_entry_provider_id(entry) {
                    let mut clause: Vec<Lit> = providers
                        .iter()
                        .filter(|&id| {
                            if let Ok(constraint) = repo.check_version_constraint(entry, id) {
                                constraint
                            } else {
                                false
                            }
                        })
                        .map(|&id| Lit::from_index(id, true))
                        .collect();
                    if clause.len() > 0 {
                        for lit in &clause {
                            if appeared.contains(&lit.index()) == false {
                                appeared.insert(lit.index());
                                q.push_back(lit.index());
                            }
                        }
                        clause.push(Lit::from_index(package_id, false));
                        formula.add_clause(&clause);
                    } else {
                        println!("version constraint not satisfied");
                        return Err(anyhow!("version constraint not satisfied"));
                    }
                }
            }
        }
        if let Some(conflicts) = repo.get_package_conflicts_by_id(package_id) {
            for entry in conflicts {
                if let Some(providers) = repo.get_entry_provider_id(entry) {
                    for provider_id in providers {
                        if *provider_id == package_id {
                            continue;
                        }
                        match repo.check_version_constraint(entry, provider_id) {
                            Ok(true) => formula.add_clause(&[
                                Lit::from_index(*provider_id, false),
                                Lit::from_index(package_id, false),
                            ]),
                            _ => continue,
                        }
                    }
                }
            }
        }
        if let Some(obsoletes) = repo.get_package_obsoletes_by_id(package_id) {
            for entry in obsoletes {
                if let Some(providers) = repo.get_entry_provider_id(entry) {
                    for provider_id in providers {
                        if *provider_id == package_id {
                            continue;
                        }
                        match repo.check_version_constraint(entry, provider_id) {
                            Ok(true) => formula.add_clause(&[
                                Lit::from_index(*provider_id, false),
                                Lit::from_index(package_id, false),
                            ]),
                            _ => continue,
                        }
                    }
                }
            }
        }
    }
    Ok(formula)
}

pub fn check_package_satisfiability_in_repo(repo: &Repo, package_name: &String) -> Result<bool> {
    if let Some(package_id) = repo.get_package_id_by_name(&package_name) {
        let formula = get_formula_by_package_id(repo, package_id)?;
        let mut solver = Solver::new();
        solver.add_formula(&formula);
        solver.assume(&[Lit::from_index(package_id, true)]);
        Ok(solver.solve()?)
    } else {
        println!(
            "Error: the package {} is not found in the repository!",
            package_name
        );
        Err(anyhow!("package not found"))
    }
}
