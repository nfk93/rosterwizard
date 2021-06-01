extern crate lp_modeler;

use std::{collections::HashMap, fs::File, io::Write, path::Path};

use lp_modeler::solvers::{CbcSolver, SolverTrait};
use lp_modeler::dsl::*;
use serde_json::*;
use serde::{Deserialize, Serialize};

// The role indices in the role table
const NROLES: usize = 16;

#[derive(Debug, Clone, Copy)]
pub enum Role {
    DH = 0,
    DK = 1,
    DR = 2,
    HU = 3,
    MA = 4,
    MO = 5,
    PA = 6,
    PR = 7,
    RO = 8,
    SH = 9,
    WA = 10,
    WL = 11,
    TANK = 12,
    HEALER = 13,
    RANGED = 14,
    MELEE = 15,
}

impl From<&str> for Role {
    fn from(s: &str) -> Self { 
        match s {
            "DH" => Self::DH,
            "DK" => Self::DK,
            "DR" => Self::DR,
            "HU" => Self::HU,
            "MA" => Self::MA,
            "MO" => Self::MO,
            "PA" => Self::PA,
            "PR" => Self::PR,
            "RO" => Self::RO,
            "SH" => Self::SH,
            "WA" => Self::WA,
            "WL" => Self::WL,
            "TANK" => Self::TANK,
            "HEALER" => Self::HEALER,
            "RANGED" => Self::RANGED,
            "MELEE" => Self::MELEE,
            _ => panic!("error reading roster json, encountered unexpected role: {}", s),
        }
    }
}
// How many bosses are there in the tier
const NBOSSES: usize = 10;

#[derive(Debug)]
pub struct Roster {
    problem: LpProblem,
    chars: Vec<Char>,
    names: Vec<String>,
}   

macro_rules! constraint_function {
    ($func_name:ident, $constraint_type:ident) => {
        pub fn $func_name (
            &mut self,
            role: Role,
            boss_requirement: &[i32],
        ) {
            for i in 0..NBOSSES {
                self.problem += self.chars()
                    .filter(|c| c.roles[role as usize])
                    .map(|c| &c.bosses[i])
                    .fold(LpExpression::from(0), |a, b| a + b)
                    .$constraint_type(boss_requirement[i]);
            }            
        }
    }
}

impl Roster {
    constraint_function!(add_role_constraint_equal, equal);
    constraint_function!(add_role_constraint_ge, ge);
    constraint_function!(add_role_constraint_le, le);

    pub fn new(roster_json: &str) -> Result<Roster> {
        let mut chars = Vec::new();
        let mut names = Vec::new();

        let parsed: Vec<Map<String, Value>> = serde_json::from_str(roster_json)?;
        let mut i = 0;
        let mut problem = LpProblem::new("setup", LpObjective::Maximize);
        
        // handle each player
        for player in parsed {
            // constraint that ensures that a player can only play a boss on one of his characters
            let mut alt_constraint: Vec<LpExpression> = (0..NBOSSES).map(|_| LpExpression::from(0)).collect();


            for (name, v) in player {
                let c: CharJson = serde_json::from_value(v)?;
                
                let roles = get_roles(&c);
                let char = Char::new(&format!("c_{}", i), roles);

                // Add boss number i to alt_constraint i for each character belonging to the player
                for (constr, var) in alt_constraint.iter_mut().zip(char.bosses.iter()) {
                    *constr += var;
                }
                
                // println!("{:?}: {:?}", name, c);
                chars.push(char);
                names.push(name);
                i += 1;
            }

            // Add all alt constraints to the problem
            for constr in alt_constraint.iter() {
                problem += constr.le(1);
            } 
            // println!("");
        }

        // Define the problem to be the sum of all the vault variables
        // let mut p = LpExpression::from(0);
        // for c in chars.iter() {
        //     for vault in c.vaults.iter() {
        //         p += vault;
        //     }
        // }

        problem += chars.iter()
            .map(|c| c.vaults.iter())
            .flatten()
            .fold(LpExpression::from(0), |a, b| a + b);

            
        // add constraint for vault decision varuables
        for c in chars.iter() {
            problem += c.bosses.iter().fold(3*&c.vaults[0], |a, b| a-b).le(0);
            problem += c.bosses.iter().fold(6*&c.vaults[1], |a, b| a-b).le(0);
            problem += c.bosses.iter().fold(9*&c.vaults[2], |a, b| a-b).le(0);
        }

        // 20 players per boss
        for i in 0..NBOSSES {
            problem += chars.iter()
                .map(|c| &c.bosses[i])
                .fold(LpExpression::from(0), |a, b| a + b)
                .equal(20);
        }

        // println!("{:#?}", problem.variables());

        // for c in chars.iter() {
        //     println!("{:?}", c);
        // }
        // println!("len of chars: {}", chars.len());

        // dummy problem
        // problem += &chars[0].vaults[0];
        // let constr1 = LpExpression::from(0) + &chars[0].bosses[1] + &chars[0].bosses[2];
        // let constr2 = LpExpression::from(0) + &chars[0].vaults[0] - &chars[0].bosses[1] -&chars[0].bosses[2];
        // problem += constr1.equal(1);
        // problem += constr2.le(0);


        let r = Roster {
            problem: problem,
            chars: chars,
            names: names,
        };

        Ok(r)
    }

    pub fn solve(&self) {
        // Specify solver
        let solver = CbcSolver::new();

        // Run optimisation and process output hashmap
        match solver.run(&self.problem) {
            Ok(solution) => {
                let mut keys: Vec<String> = solution.results.keys().map(|x| x.clone()).collect();
                keys.sort();
                for k in keys.iter() {
                    println!("{}: {}", k, solution.results.get(k).unwrap());
                }
                // let related = solution.related_problem;
                // println!("{:#?}", problem.variables());
                // println!("{:#?}", solution.results);
                println!("Status {:?}", solution.status);
                println!("Max: {:?}", solution.eval());
                // println!("{:#?}", solution.results);

                // let path = Path::new("result.txt");
                // let display = path.display();
                // let mut file = match File::create(&path) {
                //     Err(why) => panic!("couldn't create {}: {}", display, why),
                //     Ok(file) => file,
                // };
                // match file.write_all(format!("{:#?}", solution).as_bytes()) {
                //             Err(why) => panic!("couldn't write to {}: {}", display, why),
                //             Ok(_) => ()
                //         }

                // // build result map
                // let mut result: HashMap<&str, Vec<bool>> = HashMap::new();
                // for n in names.iter() {
                //     result.insert(n, vec![false; 10]);
                // }
                // for (k, v) in solution.results.iter() {
                //     let mut key_words = k.split('_');
                //     match key_words.next() {
                //         Some("player") => {
                //             let name_key = names[key_words.next().unwrap().parse::<usize>().unwrap()];
                //             let boss_idx = key_words.next().unwrap().parse::<usize>().unwrap();
                //             let vector = result.get_mut(name_key).unwrap();
                //             vector[boss_idx] = *v == 1.;
                //         }
                //         _ => () // dont do stuff for vault variables
                //     }
                // }

                // // print the result to file
                // let path = Path::new("result.txt");
                // let display = path.display();
                // let mut file = match File::create(&path) {
                //     Err(why) => panic!("couldn't create {}: {}", display, why),
                //     Ok(file) => file,
                // };
                // for name in names.iter() {
                //     let init = format!("{}", name);
                //     let mut line = result.get(name).unwrap().iter().fold(init, |l, b| l + format!("\u{0009}{}", b).as_str());
                //     line += "\n";
                //     match file.write_all(line.as_bytes()) {
                //         Err(why) => panic!("couldn't write to {}: {}", display, why),
                //         Ok(_) => ()
                //     }
                // }

            },
            Err(msg) => println!("{}", msg),
        }
    }

    fn chars<'a>(&'a self) -> impl Iterator<Item = &'a Char> {
        self.chars.iter()
    }
}

fn get_roles(char: &CharJson) -> [bool; NROLES] {
    let mut result = [false; NROLES];
    result[Role::from(char.class.as_str()) as usize] = true;
    result[Role::from(char.role.as_str()) as usize] = true;

    result
}

#[derive(Serialize, Deserialize, Debug)]
struct CharJson {
    class: String,
    main: bool,
    role: String,
}

#[derive(Debug)]
pub struct Char {
    name: String,
    bosses: [LpBinary; 10],
    vaults: [LpBinary; 3],
    roles: [bool; NROLES],
}

impl Char {
    fn new(name: &str, roles: [bool; NROLES]) -> Char {
        println!("new char: {}", name);

        Char {
            name: name.to_owned(),
            bosses: [
                LpBinary::new(format!("{}_0", name).as_str()),
                LpBinary::new(format!("{}_1", name).as_str()),
                LpBinary::new(format!("{}_2", name).as_str()),
                LpBinary::new(format!("{}_3", name).as_str()),
                LpBinary::new(format!("{}_4", name).as_str()),
                LpBinary::new(format!("{}_5", name).as_str()),
                LpBinary::new(format!("{}_6", name).as_str()),
                LpBinary::new(format!("{}_7", name).as_str()),
                LpBinary::new(format!("{}_8", name).as_str()),
                LpBinary::new(format!("{}_9", name).as_str())],
            vaults: [
                LpBinary::new(format!("{}_vault_1", name).as_str()),
                LpBinary::new(format!("{}_vault_2", name).as_str()),
                LpBinary::new(format!("{}_vault_3", name).as_str())],
            roles: roles
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use std::fs::read_to_string;

    #[test]
    fn smoke_1() {
        let path = Path::new("roster.json");
        let contents = read_to_string(path)
            .expect("Something went wrong reading the file");

        let r = Roster::new(&contents).unwrap();
        r.solve();
    }
}
