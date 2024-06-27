#![feature(test)]

extern crate test;

mod parts;
mod rocket_analysis;
mod vector;
mod kerbin;
mod integrator;

use std::fmt::Debug;

use clap::Parser;
use rand::prelude::*;

use rocket_analysis::{analyze_rocket, print_rocket_info, RocketInfo};
use parts::*;
use parts::PartVariant::*;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Number of rockets to generate
    #[arg(short, long, default_value_t = 10000)]
    count: usize,
}


fn permute_parts(base_parts: &[Part]) -> Vec<Part> {
    let mut parts = base_parts.to_vec();

    loop {
        let r = random::<f32>() % 1.0;
        if r > 0.85 {
            // Add new
            let part_type = random::<usize>() % PART_CATALOGUE.len(); 
            let part_i = random::<usize>() % (parts.len() + 1); 
            parts.insert(part_i, PART_CATALOGUE[part_type]);
        } else if r > 0.2 {
            // Replace
            if parts.len() > 0 {
                let part_i = random::<usize>() % parts.len(); 
                parts.remove(part_i);
                let part_type = random::<usize>() % PART_CATALOGUE.len();
                parts.insert(part_i, PART_CATALOGUE[part_type]);
            }
        } else {
            // Remove
            if parts.len() > 0 {
                let part_i = random::<usize>() % parts.len(); 
                parts.remove(part_i);
            }
        }
        // Allow multiple permutations to happen at once.
        if (random::<f32>() % 1.0) > 0.3 {
            break;
        }
    }
    parts
}


fn check_validity(rocket_info: &RocketInfo) -> bool {
    let final_stage = &rocket_info.stages[rocket_info.stages.len() - 1];
    let contains_command_pod = final_stage.iter().any(
        |p| matches!(p.variant, CommandPod)
    );
    let has_parachute = final_stage.iter().any(
        |p| matches!(p.variant, Parachute)
    );
    let second_stage_twr = if rocket_info.stage_info.len() > 1 {
        rocket_info.stage_info[1].twr > 0.5
    } else {
        true
    };

    rocket_info.launch_mass < 18.0 &&
    contains_command_pod &&
    has_parachute &&
    rocket_info.stage_info[0].twr > 1.5 && second_stage_twr
}


/// Returns "true" if `new_rocket` is better than `base_rocket`
fn compare_rockets(old_rocket: &RocketInfo, new_rocket: &RocketInfo) -> bool {
    if !check_validity(new_rocket) {
        return false;
    }
    if new_rocket.delta_v < old_rocket.delta_v - 0.1 {
        // We are worse in delta-V
        return false;
    }
    if (new_rocket.delta_v - old_rocket.delta_v).abs() < 0.1 &&
        new_rocket.part_count >= old_rocket.part_count {
        // We are equal in delta-V, but use more parts.
        return false;
    } 
    // We are better.
    return true;
}


fn optimize_rocket(starting_rocket: &[Part], iterations: usize) {
    let mut current_rocket: Vec<Part> = starting_rocket.to_vec();
    let mut current_info = analyze_rocket(&current_rocket);
    print_rocket_info(&current_info);

    let mut i = 0;
    while i < iterations {
        let rocket_permutation = permute_parts(&current_rocket);
        let permutation_info = analyze_rocket(&rocket_permutation);
        
        if compare_rockets(&current_info, &permutation_info)  {
            current_rocket = rocket_permutation;
            current_info = permutation_info;
            current_info.stages = sort_rocket(&current_info.stages);
            print!("i={i}, NEW STAGE: ");
            print_rocket(&current_info.stages);
            print!("\nDELTA-V: {}m/s", current_info.delta_v as i32);
            print!(" | TWR: {}", current_info.stage_info[0].twr);
            if current_info.stage_info.len() > 1 {
                print!(" | TWR (2. STAGE): {}", current_info.stage_info[1].twr);
            }
            print!("\n\n");
        }
        i += 1;
    }

    print_rocket_info(&current_info);
    println!("FINAL DELTA-V: {}m/s", current_info.delta_v as i32);
}


#[bench]
fn benchmark(b: &mut test::Bencher) {
    b.iter(|| optimize_rocket(DEFAULT_ROCKET_1, 10000));
}


fn main() {
    let args = Args::parse();

    optimize_rocket(DEFAULT_ROCKET_1, args.count);
}
 