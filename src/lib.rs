extern crate lp_modeler;

use std::{collections::HashMap, fs::File, io::Write, path::Path};

use lp_modeler::solvers::{CbcSolver, SolverTrait};
use lp_modeler::dsl::*;

macro_rules! generate_constraint_function {
    ($func_name:ident, $constraint_type:ident) => {
        fn $func_name (
            mut problem: LpProblem,
            characters: &Vec<[LpBinary; 10]>,
            is_role: &[i32],
            boss_requirement: &[i32],
        ) -> LpProblem {
            for i in 0..10 {
                problem += characters.iter().map(|player_assignments| {
                    &player_assignments[i]
                })
                .zip(is_role.iter())
                .filter(|(_, player_is_role)| **player_is_role == 1i32)
                .fold(LpExpression::from(0), |a, (b, _)| a + b)
                .$constraint_type(boss_requirement[i]);
            }
        
            return problem
        }
    }
}

generate_constraint_function!(add_role_constraint_equal, equal);
// generate_constraint_function!(add_role_constraint_le, le);
generate_constraint_function!(add_role_constraint_ge, ge);

// Precondition: boss_healers and boss_tanks has the correct length (10)
fn add_boss_constraints(
    mut problem: LpProblem, 
    boss_healers: &[i32],
    boss_tanks: &[i32],
    characters: &Vec<[LpBinary; 10]>,
    is_healer: &[i32],
    is_tank: &[i32],
    is_dh: &[i32],
    is_monk: &[i32],
    is_pala: &[i32],
    is_mage: &[i32],
    is_priest: &[i32],
    is_warr: &[i32],
    is_warl: &[i32],
) -> LpProblem {

    // must have 20 players
    for i in 0..10 {
        problem += characters.iter().map(|player_assignments| {
            &player_assignments[i]
        })
        .fold(LpExpression::from(0), |a, b| a + b)
        .equal(20);
    }

    problem = add_role_constraint_equal(problem, characters, is_healer, boss_healers);
    problem = add_role_constraint_equal(problem, characters, is_tank, boss_tanks);

    let unit_req = [1; 10];
    problem = add_role_constraint_ge(problem, characters, is_dh, &unit_req);
    problem = add_role_constraint_ge(problem, characters, is_monk, &unit_req);
    problem = add_role_constraint_ge(problem, characters, is_pala, &unit_req);
    problem = add_role_constraint_ge(problem, characters, is_mage, &unit_req);
    problem = add_role_constraint_ge(problem, characters, is_priest, &unit_req);
    problem = add_role_constraint_ge(problem, characters, is_warr, &unit_req);
    problem = add_role_constraint_ge(problem, characters, is_warl, &unit_req);

    return problem
}

pub fn run(
    names: &[&str],
    boss_healers: &[i32],
    boss_tanks: &[i32],
    is_healer: &[i32],
    is_tank: &[i32],
    is_dh: &[i32],
    is_monk: &[i32],
    is_pala: &[i32],
    is_mage: &[i32],
    is_priest: &[i32],
    is_warr: &[i32],
    is_warl: &[i32]
) {
    let n = 25usize;

    let characters: Vec<[LpBinary; 10]> = (0..n).map(|i| {
        [
            LpBinary::new(format!("player_{}_0", i).as_str()),
            LpBinary::new(format!("player_{}_1", i).as_str()),
            LpBinary::new(format!("player_{}_2", i).as_str()),
            LpBinary::new(format!("player_{}_3", i).as_str()),
            LpBinary::new(format!("player_{}_4", i).as_str()),
            LpBinary::new(format!("player_{}_5", i).as_str()),
            LpBinary::new(format!("player_{}_6", i).as_str()),
            LpBinary::new(format!("player_{}_7", i).as_str()),
            LpBinary::new(format!("player_{}_8", i).as_str()),
            LpBinary::new(format!("player_{}_9", i).as_str())            
        ]
    }).collect();

    let vaults: Vec<_> = (0..n).map(|i| {
        [
            LpBinary::new(format!("vault_{}_1", i).as_str()),
            LpBinary::new(format!("vault_{}_2", i).as_str()),
            LpBinary::new(format!("vault_{}_3", i).as_str())
        ]
    }).collect();

    
    
    let mut problem = LpProblem::new("Decide setup", LpObjective::Maximize);
    problem += vaults.iter().flatten().fold(LpExpression::from(0), |a, b| a + b);
    
    problem = add_boss_constraints(
        problem, 
        &boss_healers, 
        &boss_tanks, 
        &characters, 
        &is_healer, 
        &is_tank, 
        &is_dh, 
        &is_monk, 
        &is_pala, 
        &is_mage, 
        &is_priest, 
        &is_warr, 
        &is_warl
    );

    // vault variables
    for i in 0..n {
        problem += characters[i].iter().fold(3*&vaults[i][0], |a, b| a-b).le(0);
        problem += characters[i].iter().fold(6*&vaults[i][1], |a, b| a-b).le(0);
        problem += characters[i].iter().fold(9*&vaults[i][2], |a, b| a-b).le(0);
    }

    // Specify solver
    let solver = CbcSolver::new();

    // Run optimisation and process output hashmap
    match solver.run(&problem) {
        Ok(solution) => {
            println!("Status {:?}", solution.status);
            let mut keys: Vec<String> = solution.results.keys().map(|x| x.clone()).collect();
            keys.sort();
            for k in keys.iter() {
                println!("{}: {}", k, solution.results.get(k).unwrap());
            }
            println!("Max: {:?}", solution.eval());

            // build result map
            let mut result: HashMap<&str, Vec<bool>> = HashMap::new();
            for n in names.iter() {
                result.insert(n, vec![false; 10]);
            }
            for (k, v) in solution.results.iter() {
                let mut key_words = k.split('_');
                match key_words.next() {
                    Some("player") => {
                        let name_key = names[key_words.next().unwrap().parse::<usize>().unwrap()];
                        let boss_idx = key_words.next().unwrap().parse::<usize>().unwrap();
                        let vector = result.get_mut(name_key).unwrap();
                        vector[boss_idx] = *v == 1.;
                    }
                    _ => () // dont do stuff for vault variables
                }
            }

            // print the result to file
            let path = Path::new("result.txt");
            let display = path.display();
            let mut file = match File::create(&path) {
                Err(why) => panic!("couldn't create {}: {}", display, why),
                Ok(file) => file,
            };
            for name in names.iter() {
                let init = format!("{}", name);
                let mut line = result.get(name).unwrap().iter().fold(init, |l, b| l + format!("\u{0009}{}", b).as_str());
                line += "\n";
                match file.write_all(line.as_bytes()) {
                    Err(why) => panic!("couldn't write to {}: {}", display, why),
                    Ok(_) => ()
                }
            }

        },
        Err(msg) => println!("{}", msg),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const NAMES: [&str; 30] = [
        "Tamica",
        "Tuyet",
        "Lezlie",
        "Gilberto",
        "Betty",
        "Robyn",
        "Carie",
        "Chanel",
        "Dakota",
        "Malvina",
        "Lorna",
        "Dede",
        "Dinorah",
        "Jeraldine",
        "Stevie",
        "Sharen",
        "Natashia",
        "Else",
        "Dora",
        "Elsy",
        "Tennie",
        "Lauralee",
        "Dorethea",
        "Dalila",
        "Lucia",
        "Kayleigh",
        "Felisha",
        "Pura",
        "Iesha",
        "Jewell"];

    #[test]
    fn test_optimization() {
        let _n = 25usize;

        let names: Vec<&str> = NAMES.iter().take(25).map(|x| *x).collect();

        let ref boss_healers = [3, 4, 4, 3, 4, 5, 4, 5, 5, 4];
        let ref boss_tanks = [2, 2, 2, 2, 2, 2, 2, 2, 2, 2];

        let ref is_healer = [0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let ref is_tank =   [1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let ref is_dh =     [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let ref is_monk =   [1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let ref is_pala =   [0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let ref is_mage =   [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let ref is_priest = [0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let ref is_warr =   [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let ref is_warl =   [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        run(&names, boss_healers, boss_tanks, is_healer, is_tank, is_dh, is_monk, is_pala, is_mage, is_priest, is_warr, is_warl);
    }
}
