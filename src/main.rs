// hashmap and hashset
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Instant;
// Combinations
use itertools::Itertools;
use rayon::prelude::*;
// serde json
use serde_json::json;
// serialize
use serde::{Deserialize, Serialize};
// _file reading and writing
use std::fs;
use std::env;
use std::fs::File;
use std::io::Write;


fn read_champs(_file: &str) -> (HashMap<u8, String>, HashMap<String, u8>) {
    // champs is a json file containing a list of all champions
    let data = fs::read_to_string(_file).expect("Something went wrong reading the file");
    // parse the json file
    let serde_data: Vec<serde_json::Value> = serde_json::from_str(&data).unwrap();
    // filter out champs with 0 cost
    let champs: Vec<String> = serde_data.iter().filter(|x| x["cost"].as_u64().unwrap() > 0).map(|x| x["name"].as_str().unwrap().to_string()).collect();
    let mut champs_map: HashMap<u8, String> = HashMap::new();
    let mut champs_map_rev: HashMap<String, u8> = HashMap::new();
    for (i, champ) in champs.iter().enumerate() {
        champs_map.insert(i as u8, champ.to_string());
        champs_map_rev.insert(champ.to_string(), i as u8);
    }
    (champs_map, champs_map_rev)
}
fn read_costs(_file: &str, champs_rev: &HashMap<String, u8>) -> HashMap<u8, u8> {
    // reads the cost of each champion from the _file
    // returns:
    // a hashmap mapping integers to the cost of the champions
    let data = fs::read_to_string(_file).expect("Something went wrong reading the file");
    let serde_data: Vec<serde_json::Value> = serde_json::from_str(&data).unwrap();
    // filter out the champs with 0 cost
    let costs:Vec<u8> = serde_data.iter().filter(|x| x["cost"].as_u64().unwrap() > 0).map(|x| x["cost"].as_u64().unwrap() as u8).collect();
    // create a hashmap of the costs
    let mut costs_map: HashMap<u8, u8> = HashMap::new();
    for (i, cost) in costs.iter().enumerate() {
        costs_map.insert(i as u8, *cost);
    }
    costs_map
}
fn read_traits(_file: &str) -> (HashMap<u8, String>, HashMap<String, u8>) {
    // reads the traits from the _file
    // returns:
    // 1. a hashmap mapping integers to the names of the traits
    // 2. a hashmap mapping names of the traits to their integer
    let data = fs::read_to_string(_file).expect("Something went wrong reading the file");
    let serde_data: Vec<serde_json::Value> = serde_json::from_str(&data).unwrap();
    // we are after just the name attribute of each trait
    let traits: Vec<String> = serde_data.iter().map(|x| x["name"].as_str().unwrap().to_string()).collect();
    // create a hashmap of the traits
    let mut traits_map: HashMap<u8, String> = HashMap::new();
    let mut traits_map_rev: HashMap<String, u8> = HashMap::new();
    for (i, atrait) in traits.iter().enumerate() {
        traits_map.insert(i as u8, atrait.to_string());
        traits_map_rev.insert(atrait.to_string(), i as u8);
    }
    (traits_map, traits_map_rev)
}
fn read_breaks(_file: &str, traits: &HashMap<String, u8>) -> HashMap<u8, HashSet<u8>> {
    // reads the breaks from the _file
    // returns:
    // 1. a hashmap mapping traits (by id) to their breakpoints

    let data = fs::read_to_string(_file).expect("Something went wrong reading the file");
    let serde_data: Vec<serde_json::Value> = serde_json::from_str(&data).unwrap();
    // each trait has a name and a list of breakpoints. we map the name to the id using traits
    let mut breaks: HashMap<u8, HashSet<u8>> = HashMap::new();
    for atrait in serde_data.iter() {
        let trait_name = atrait["name"].as_str().unwrap();
        let trait_id = traits[trait_name];
        let trait_breaks: Vec<u8> = atrait["breaks"].as_array().unwrap().iter().map(|x| x.as_u64().unwrap() as u8).collect();
        let mut trait_breaks_set: HashSet<u8> = trait_breaks.iter().cloned().collect();
        trait_breaks_set.insert(0);
        breaks.insert(trait_id, trait_breaks_set);
    }
    breaks
}
fn read_champ_traits(_file: &str, champs_rev: &HashMap<String, u8>, traits_rev: &HashMap<String, u8>) -> HashMap<u8, Vec<u8>> {
    // reads the champ_traits from the _file
    // returns:
    // 1. a hashmap mapping champions (by id) to their traits

    let data = fs::read_to_string(_file).expect("Something went wrong reading the file");
    let serde_data: Vec<serde_json::Value> = serde_json::from_str(&data).unwrap();
    // each champ has a "traits" key which is a list of trait names. we map the name to the id using traits
    // we also map the champ name to id using champs
    let mut champ_traits: HashMap<u8, Vec<u8>> = HashMap::new();
    for champ in serde_data.iter() {
        // if the cost is > 0, we have a valid champ
        if champ["cost"].as_u64().unwrap() > 0 {
            let champ_name = champ["name"].as_str().unwrap();
            let champ_id = champs_rev[champ_name];
            let trait_names: Vec<String> = champ["traits"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect();
            let trait_ids: Vec<u8> = trait_names.iter().map(|x| traits_rev[x]).collect();
            champ_traits.insert(champ_id, trait_ids);
        }
    }
    champ_traits
}
fn less_than_n_wasted(team: &Vec<&u8>, traits: &HashMap<u8, String>, unit_traits: &HashMap<u8, Vec<u8>>,wastes: &HashMap<u8, HashMap<u8, u8>>, n: &u8) -> bool {
    // checks if the team has less than n wasted traits
    let mut team_traits = HashMap::new();
    for trait_id in traits.keys() {
        team_traits.insert(*trait_id, 0);
    }
    for nit in team {
        // for each unit in the team get the traits
        let unit_traits = &unit_traits[*nit];
        // increment the count for each trait
        for trait_id in unit_traits {
            team_traits.insert(*trait_id, team_traits[trait_id] + 1);
        }
    }
    // for each trait in the team
    // if the count is not in the breaks for the trait
    // return false
    let mut waste_count = 0;
    for (trait_id, count) in team_traits.iter_mut() {
        // for each trait in the team get the waste for the count
        waste_count += wastes[trait_id][count];
    }
    if waste_count <= *n {
        return true;
    }
    false
}
// serde serialization
#[derive(Serialize, Deserialize)]
struct Team {
    size: u8,
    // a team is a vector of champions
    team: Vec<String>,
    // the active traits of the team and their level
    active_traits: HashMap<String, u8>,
    // the traits wasted by the team 
    wasted_traits: HashMap<String, u8>,
    // total traits wasted by the team
    total_wasted_traits: u8,
    // the total cost of the team
    total_cost: u8,
    // the highest cost unit in the team (tuple of name and cost)
    max_cost: (String, u8),
    // the lowest cost unit in the team (tuple of name and cost)
    min_cost: (String, u8),
    // the average cost of the team
    average_cost: f64,
}

impl Team {
    fn new(team: Vec<String>, size: u8, active_traits: HashMap<String, u8>, wasted_traits: HashMap<String, u8>, total_wasted_traits: u8, total_cost: u8, max_cost: (String, u8), min_cost: (String, u8), average_cost: f64) -> Team {
        Team {
            team,
            size,
            active_traits,
            wasted_traits,
            total_wasted_traits,
            total_cost,
            max_cost,
            min_cost,
            average_cost,
        }
    }
    fn get_team_traits(active_traits: &mut HashMap<String, u8>, wasted_traits: &mut HashMap<String, u8>, total_waste: &mut u8, team: &Vec<&u8>, traits: &HashMap<u8, String>, unit_traits: &HashMap<u8, Vec<u8>>, wastes: &HashMap<u8, HashMap<u8, u8>>) -> u8 {
        let mut team_traits = HashMap::new();
        for trait_id in traits.keys() {
            team_traits.insert(*trait_id, 0);
        }
        for nit in team {
            let unit_traits = &unit_traits[*nit];
            for trait_id in unit_traits {
                team_traits.insert(*trait_id, team_traits[trait_id] + 1);
            }
        }
        let mut waste_count = 0;
        for (trait_id, count) in team_traits.iter_mut() {
            let this_waste = wastes[trait_id][count];
            waste_count += this_waste;
            if this_waste > 0 {
                wasted_traits.insert(traits[trait_id].clone(), this_waste);
            }
            if *count != this_waste || (count > &mut 0 && this_waste == 0) {
                active_traits.insert(traits[trait_id].clone(), *count - this_waste);
            }
        }
        *total_waste = waste_count;
        return 0;
    }
    fn get_team_costs(total_cost: &mut u8, max_cost: &mut (String, u8), min_cost: &mut (String, u8), average_cost: &mut f64, team: &Vec<&u8>, champs: &HashMap<u8, String>, costs: &HashMap<u8, u8>) -> u8 {
        // get the cost of the team
        // return the total cost
        let mut team_cost = 0;
        for nit in team {
            // for each unit in the team
            // get the cost of the unit
            let cost = costs[*nit];
            // add the cost to the team cost
            team_cost += cost;
            // if the cost is greater than the max cost
            if cost > max_cost.1 {
                // record the new max cost
                max_cost.0 = champs[*nit].to_string();
                max_cost.1 = cost;
            }
            // if the cost is less than the min cost
            if cost < min_cost.1 {
                // record the new min cost
                min_cost.0 = champs[*nit].to_string();
                min_cost.1 = cost;
            }
        }
        // record the average cost and team cost
        *average_cost = team_cost as f64 / team.len() as f64;
        
        *total_cost = team_cost;
        // return success
        return 0;
    }
    fn team_from_list(team: &Vec<&u8>, traits: &HashMap<u8, String>,  champs: &HashMap<u8, String>, unit_traits: &HashMap<u8, Vec<u8>>, wastes: &HashMap<u8, HashMap<u8, u8>>, costs: &HashMap<u8, u8>) -> Team {
        let size = team.len() as u8;
        let mut str_team = team.iter().map(|&nit| champs[nit].to_string()).collect::<Vec<String>>();
        let mut active_traits = HashMap::new();
        let mut wasted_traits = HashMap::new();
        let mut total_wasted_traits = 0;

        Team::get_team_traits(&mut active_traits, &mut wasted_traits, &mut total_wasted_traits, &team, &traits, &unit_traits, &wastes);
        let mut total_cost = 0;
        let mut max_cost = ("".to_string(), 0);
        let mut min_cost = ("".to_string(), 6);
        let mut average_cost = 0.0;
        Team::get_team_costs(&mut total_cost, &mut max_cost, &mut min_cost, &mut average_cost, &team, &champs, &costs);
        return Team::new(str_team, size, active_traits, wasted_traits, total_wasted_traits, total_cost, max_cost, min_cost, average_cost);
    }
}


fn do_ltn_synergies(champs: &HashMap<u8, String>, traits: &HashMap<u8, String>, unit_traits: &HashMap<u8, Vec<u8>>, wastes: &HashMap<u8, HashMap<u8, u8>>, costs: &HashMap<u8, u8>, teamsize: &u8, n: &u8) -> Vec<Team> {
    // returns:
    // a tuple of the following:
    // 1 teamsize, 2 a vector of the teams of that size that have less than n wasted traits

    // initialize the vector of teams
    let teams: Vec<Team> = Vec::new();
    // .par_bridge()
    let ps = champs.keys().combinations((*teamsize).into()).par_bridge().filter({
        |team|
        less_than_n_wasted(team, traits, unit_traits, wastes, n)
    }).map({
        |team|
        Team::team_from_list(&team, &traits, &champs, &unit_traits, &wastes, &costs)
    }).collect::<Vec<Team>>();
    return ps;
}

fn do_all_ltn_synergies(champs: &HashMap<u8, String>, traits: &HashMap<u8, String>, champtraits: &HashMap<u8, Vec<u8>>,wastes: &HashMap<u8, HashMap<u8, u8>>, costs: &HashMap<u8, u8>, min_teamsize: &u8, max_teamsize: &u8, n: &u8) -> Vec<Team> {
    // does less than n wasted synergies for all teamsizes between min_teamsize and max_teamsize
    // returns:
    // a hashmap mapping teamsize to a vector of teams
    // n is the maximum number of wasted traits
    let mut teams = Vec::new();
    // for each teamsize
    for teamsize in *min_teamsize..=*max_teamsize {
        let now = Instant::now();
        // do the ltn synergies for the teamsize
        let ps = do_ltn_synergies(&champs, &traits, &champtraits, &wastes, &costs, &teamsize, &n);
        // extend the list with the new teams
        teams.extend(ps);
        // print the time it took
        let elapsed = now.elapsed();
        // print elapsed time
        //println!("Finished teamsize {} in {}.{:09} seconds", teamsize, elapsed.as_secs(), elapsed.subsec_nanos());
    }
    return teams;
}
fn synergies_to_json(teams: &Vec<Team>, file: &str) {
    let mut file = File::create(file).unwrap();
    let json_teams = json!(teams);
    let pretty_json = serde_json::to_string_pretty(&json_teams).unwrap();
    // write the string to the file
    file.write_all(pretty_json.as_bytes()).unwrap();
}
fn compute_wastes(breaks: &HashMap<u8, HashSet<u8>>) -> HashMap<u8, HashMap<u8, u8>> {
    // returns:
    // a hashmap mapping a trait to the wasted traits for each count of that trait

    let mut wastes = HashMap::new();
    // for each trait
    for trait_ in breaks.keys() {
        // initialize the wasted traits for that trait
        let mut wasted_traits = HashMap::new();
        // for each possible number of that trait from  1 to 9 (overkill but whatever)
        for count in 0..=9 {
            // waste for a count is count-(largest number in the breaks hashset <= count)
            wasted_traits.insert(count, count - breaks.get(trait_).unwrap().iter().filter(|x| **x <= count).max().unwrap());
        }
        // add the wasted traits to the hashmap
        wastes.insert(*trait_, wasted_traits);
}
    wastes
}
fn main () {
    // args:
    // 1: the output folder
    // 2: the maximum waste
    // 3: the minimum teamsize
    // 4: the maximum teamsize
    let args: Vec<String> = env::args().collect();

    // produces the following
    // hashmap champs int, string where the ints are champid and the strings are names
    // hashmap costs string, int where the strings are champ names and the ints are the costs
    // hashmap traits int, string where the ints are trait ids and the strings are trait names
    // hashmap breaks int, HashSet<int> where the ints are trait ids and the set is the set of breakpoints
    // champtraits hashmap int, hashset<int> where the ints are champ ids and the set is the set of trait ids for the champ
    // handle the arguments
    if args.len() != 7 {
        println!("Usage: {} <output folder> <max waste> <min teamsize> <max teamsize> <champs_filename> <traits_filename>", args[0]);
        return;
    }
    let out_folder = &args[1];
    let n = args[2].parse::<u8>().unwrap();
    let min_team_size = args[3].parse::<u8>().unwrap();
    let max_team_size = args[4].parse::<u8>().unwrap();
    let champs_filename = args[5].to_string();
    let traits_filename = args[6].to_string();

    let (champs, champs_rev) = read_champs(&champs_filename);
    let costs = read_costs(&champs_filename, &champs_rev);
    let traits = read_traits(&traits_filename);
    let (traits, traits_rev) = read_traits(&traits_filename);
    let breaks = read_breaks(&traits_filename, &traits_rev);
    let champ_traits = read_champ_traits(&champs_filename, &champs_rev, &traits_rev);
    let wastes = compute_wastes(&breaks);

    let teams = do_all_ltn_synergies(&champs, &traits, &champ_traits, &wastes, &costs, &min_team_size, &max_team_size, &n);
    let orig_ts = champs_filename.split("/").last().unwrap().split("_").last().unwrap().split(".").next().unwrap();
    let fname = format!("{}/teams_sizes_{}_to_{}_max_waste_{}_{}.json", out_folder, min_team_size, max_team_size, n, orig_ts);
    synergies_to_json(&teams, &fname);
}
