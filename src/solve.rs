use crate::repo::{IdT, Repo};
use anyhow::{anyhow, Result};
use std::collections::{HashSet, VecDeque};
use varisat::{CnfFormula, ExtendFormula, Lit, solver::Solver};

fn get_formula_by_package_id(repo: &Repo, package_id: IdT) -> Result<CnfFormula> {
    let mut q = VecDeque::new();
    let mut formula = CnfFormula::new();
    let mut appeared = HashSet::new();
    q.push_back(package_id);
    while let Some(package_id) = q.pop_front() {
        if let Some(requires) = repo.get_package_requires_by_id(package_id) {
            for entry in requires {
                if let Some(providers) = repo.get_entry_provider_id(entry) {
                    let mut clause: Vec<Lit> = providers
                        .into_iter()
                        .map(|id| Lit::from_index(*id, true))
                        .collect();
                    clause.push(Lit::from_index(package_id, false));
                    formula.add_clause(&clause);
                    for provider_id in providers {
                        if appeared.contains(provider_id) == false {
                            appeared.insert(provider_id);
                            q.push_back(*provider_id);
                        }
                    }
                }
            }
        }
        if let Some(conflicts) = repo.get_package_conflicts_by_id(package_id) {
            for entry in conflicts {
                if let Some(providers) = repo.get_entry_provider_id(entry) {
                    let mut clause: Vec<Lit> = providers
                        .into_iter()
                        .map(|id| Lit::from_index(*id, true))
                        .collect();
                    clause.push(Lit::from_index(package_id, false));
                    formula.add_clause(&clause);
                    for provider_id in providers {
                        if appeared.contains(provider_id) == false {
                            appeared.insert(provider_id);
                            q.push_back(*provider_id);
                        }
                    }
                }
            }
        }
        if let Some(obsoletes) = repo.get_package_obsoletes_by_id(package_id) {
            for entry in obsoletes {
                if let Some(providers) = repo.get_entry_provider_id(entry) {
                    let mut clause: Vec<Lit> = providers
                        .into_iter()
                        .map(|id| Lit::from_index(*id, false))
                        .collect();
                    clause.push(Lit::from_index(package_id, false));
                    formula.add_clause(&clause);
                    for provider_id in providers {
                        if appeared.contains(provider_id) == false {
                            appeared.insert(provider_id);
                            q.push_back(*provider_id);
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
        Ok(solver.solve()?)
    } else {
        Err(anyhow!("The package {} is not found in the repository!", package_name))
    }
}