use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Instant;
use itertools::Itertools;
use rayon::prelude::*;
use serde_json::json;
use serde::{Deserialize, Serialize};
use std::fs;
use std::env;
use std::fs::File;
use std::io::Write;


fn read_champs(_file: &str) -> (HashMap<u8, String>, HashMap<String, u8>) {
    let data = fs::read_to_string(_file).expect("Something went wrong reading the file");
    let serde_data: Vec<serde_json::Value> = serde_json::from_str(&data).unwrap();
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
    let data = fs::read_to_string(_file).expect("Something went wrong reading the file");
    let serde_data: Vec<serde_json::Value> = serde_json::from_str(&data).unwrap();
    let costs:Vec<u8> = serde_data.iter().filter(|x| x["cost"].as_u64().unwrap() > 0).map(|x| x["cost"].as_u64().unwrap() as u8).collect();
    let mut costs_map: HashMap<u8, u8> = HashMap::new();
    for (i, cost) in costs.iter().enumerate() {
        costs_map.insert(i as u8, *cost);
    }
    costs_map
}
fn read_traits(_file: &str) -> (HashMap<u8, String>, HashMap<String, u8>) {
    let data = fs::read_to_string(_file).expect("Something went wrong reading the file");
    let serde_data: Vec<serde_json::Value> = serde_json::from_str(&data).unwrap();
    let traits: Vec<String> = serde_data.iter().map(|x| x["name"].as_str().unwrap().to_string()).collect();
    let mut traits_map: HashMap<u8, String> = HashMap::new();
    let mut traits_map_rev: HashMap<String, u8> = HashMap::new();
    for (i, atrait) in traits.iter().enumerate() {
        traits_map.insert(i as u8, atrait.to_string());
        traits_map_rev.insert(atrait.to_string(), i as u8);
    }
    (traits_map, traits_map_rev)
}
fn read_breaks(_file: &str, traits: &HashMap<String, u8>) -> HashMap<u8, HashSet<u8>> {
    let data = fs::read_to_string(_file).expect("Something went wrong reading the file");
    let serde_data: Vec<serde_json::Value> = serde_json::from_str(&data).unwrap();
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
    let data = fs::read_to_string(_file).expect("Something went wrong reading the file");
    let serde_data: Vec<serde_json::Value> = serde_json::from_str(&data).unwrap();
    let mut champ_traits: HashMap<u8, Vec<u8>> = HashMap::new();
    for champ in serde_data.iter() {
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
        waste_count += wastes[trait_id][count];
    }
    if waste_count <= *n {
        return true;
    }
    false
}
#[derive(Serialize, Deserialize)]
struct Team {
    size: u8,
    team: Vec<String>,
    active_traits: HashMap<String, u8>,
    wasted_traits: HashMap<String, u8>,
    total_wasted_traits: u8,
    total_cost: u8,
    max_cost: (String, u8),
    min_cost: (String, u8),
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
        let mut team_cost = 0;
        for nit in team {
          
            let cost = costs[*nit];
            team_cost += cost;
            if cost > max_cost.1 {
                max_cost.0 = champs[*nit].to_string();
                max_cost.1 = cost;
            }
            if cost < min_cost.1 {
                min_cost.0 = champs[*nit].to_string();
                min_cost.1 = cost;
            }
        }
        *average_cost = team_cost as f64 / team.len() as f64;
        
        *total_cost = team_cost;
        return 0;
    }
    fn unit_with_min_cost(team: &Vec<&u8>, costs: &HashMap<u8, u8>, champs: &HashMap<u8, String>) -> (String, u8) {
        let mut min_cost = (String::from(""), 100);
        for nit in team {
            let cost = costs[*nit];
            if cost < min_cost.1 {
                min_cost.0 = champs[*nit].to_string();
                min_cost.1 = cost;
            }
        }
        min_cost
    }
    fn unit_with_max_cost(team: &Vec<&u8>, costs: &HashMap<u8, u8>, champs: &HashMap<u8, String>) -> (String, u8) {
        let mut max_cost = (String::from(""), 0);
        for nit in team {
            let cost = costs[*nit];
            if cost > max_cost.1 {
                max_cost.0 = champs[*nit].to_string();
                max_cost.1 = cost;
            }
        }
        max_cost
    }
    fn team_from_list(team: &Vec<&u8>, traits: &HashMap<u8, String>,  champs: &HashMap<u8, String>, unit_traits: &HashMap<u8, Vec<u8>>, wastes: &HashMap<u8, HashMap<u8, u8>>, costs: &HashMap<u8, u8>) -> Team {
        let size = team.len() as u8;
        let mut str_team = team.iter().map(|&nit| champs[nit].to_string()).collect::<Vec<String>>();
        let mut active_traits = HashMap::new();
        let mut wasted_traits = HashMap::new();
        let mut total_wasted_traits = 0;

        Team::get_team_traits(&mut active_traits, &mut wasted_traits, &mut total_wasted_traits, &team, &traits, &unit_traits, &wastes);
        let mut total_cost = 0;
        let mut max_cost = unit_with_max_cost(&team, &costs, &champs);
        let mut min_cost = unit_with_min_cost(&team, &costs, &champs);
        let mut average_cost = 0.0;
        Team::get_team_costs(&mut total_cost, &mut max_cost, &mut min_cost, &mut average_cost, &team, &champs, &costs);
        return Team::new(str_team, size, active_traits, wasted_traits, total_wasted_traits, total_cost, max_cost, min_cost, average_cost);
    }
}


fn do_ltn_synergies(champs: &HashMap<u8, String>, traits: &HashMap<u8, String>, unit_traits: &HashMap<u8, Vec<u8>>, wastes: &HashMap<u8, HashMap<u8, u8>>, costs: &HashMap<u8, u8>, teamsize: &u8, n: &u8) -> Vec<Team> {
    let teams: Vec<Team> = Vec::new();
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
    let mut teams = Vec::new();
    for teamsize in *min_teamsize..=*max_teamsize {
        let now = Instant::now();
        let ps = do_ltn_synergies(&champs, &traits, &champtraits, &wastes, &costs, &teamsize, &n);
        teams.extend(ps);
        let elapsed = now.elapsed();
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
   

    let mut wastes = HashMap::new();
    for trait_ in breaks.keys() {
        let mut wasted_traits = HashMap::new();
        for count in 0..=9 {
            wasted_traits.insert(count, count - breaks.get(trait_).unwrap().iter().filter(|x| **x <= count).max().unwrap());
        }
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
