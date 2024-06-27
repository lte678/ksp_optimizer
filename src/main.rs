#![feature(test)]

extern crate test;

mod parts;
mod vector;
mod kerbin;
mod integrator;

use std::fmt::Debug;

use clap::Parser;
use rand::prelude::*;

use crate::vector::Vector;
use crate::parts::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Number of rockets to generate
    #[arg(short, long, default_value_t = 10000)]
    count: usize,
}

const GRAVITY: f32 = 9.81;


fn get_burnout_times(stage: &[Part]) -> Vec<f32> {
    let mut burnout_times = Vec::new();
    let fuel_mass = get_part_fuel(stage) * EFF_FUEL_DENSITY;
    let mut liquid_mass_flow: f32 = 0.0;
    for part in stage {
        if let Part::SolidBooster{ fuel, thrust_asl, isp_asl, ..} = part {
            let solid_fuel = *fuel * SOLID_FUEL_DENSITY;
            let solid_mass_flow = thrust_asl / (isp_asl * GRAVITY);
            if solid_fuel > 1e-6 && solid_mass_flow > 1e-6 {
                burnout_times.push(solid_fuel / solid_mass_flow);
            }
        }
        if let Part::Engine{ thrust_asl, isp_asl, .. } = part {
            liquid_mass_flow += thrust_asl / (isp_asl * GRAVITY);
        }
    }
    if liquid_mass_flow > 1e-6 && fuel_mass > 1e-6 {
        burnout_times.push(fuel_mass / liquid_mass_flow);
    }
    
    burnout_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    burnout_times
}


fn flight_dynamics(t: f32, state: &Vector<3>, stage: &[Part], payload_mass: f32) -> Vector<3> {
    // The state consists of delta-velocity, velocity and height
    let [_, v, altitude] = state.data;

    let fuel_mass = get_part_fuel(stage) * EFF_FUEL_DENSITY;

    let mut liquid_thrust_asl: f32 = 0.0;
    let mut liquid_thrust_vac: f32 = 0.0;
    let mut solid_thrust: f32 = 0.0;
    let mut liquid_mass_flow: f32 = 0.0;
    let mut solid_rockets: Vec<(f32, f32, f32, f32)> = Vec::new();
    for part in stage {
        if let Part::SolidBooster{ fuel, thrust_asl, thrust_vac, isp_asl, ..} = part {
            solid_rockets.push((*fuel * SOLID_FUEL_DENSITY, *thrust_asl, *thrust_vac, thrust_asl / (isp_asl * GRAVITY)));
            solid_thrust += thrust_asl;
        }
        if let Part::Engine{ thrust_asl,  thrust_vac, isp_asl, .. } = part {
            liquid_thrust_asl += thrust_asl;
            liquid_thrust_vac += thrust_vac;
            liquid_mass_flow += thrust_asl / (isp_asl * GRAVITY)
        }
    }

    let atmo_p = kerbin::get_pressure(altitude);
    let mut thrust = 0.0;
    let mut mass = payload_mass + get_stage_mass_wet(stage);
    if fuel_mass > liquid_mass_flow * t && liquid_thrust_asl > 1e-6 {
        thrust += liquid_thrust_asl * atmo_p + liquid_thrust_vac * (1.0 - atmo_p);
    }
    mass -= fuel_mass.min(liquid_mass_flow * t);
    
    for (s_fuel_mass, s_thrust, s_thrust_vac, s_mass_flow) in &mut solid_rockets {
        if *s_fuel_mass > *s_mass_flow * t && *s_thrust > 1e-6 {
            thrust += *s_thrust * atmo_p + *s_thrust_vac * (1.0 - atmo_p);
        }
        mass -= s_fuel_mass.min(*s_mass_flow * t);
    }

    let a = thrust / mass;
    let a_real = a - GRAVITY;
    Vector{data: [a, a_real, v]}
}


fn integrate_dv2(stage: &[Part], payload_mass: f32, altitude: f32, velocity: f32) -> (f32, f32, f32, f32) {
    let f = |t, state| flight_dynamics(t, &state, stage, payload_mass);

    let mut times = get_burnout_times(stage);
    times.insert(0, 0.0);

    let mut delta_v = 0.0;
    let mut velocity = 0.0;
    let mut altitude = 0.0;

    for t_i in 0..times.len()-1 {
        let (res, res_info) = integrator::rk45(
            &f, 
            Vector{ data: [0.0, velocity, altitude] },
            times[t_i]+1e-6, times[t_i+1]-1e-3,
            Vector{ data: [1e-3, 1e-9, 1e-9]},
            1e-4
        );
        delta_v += res[0];
        velocity += res[1];
        altitude += res[2];
    }

    // Get thrust information
    let mut thrust = 0.0;
    for part in stage {
        if let Part::SolidBooster{ thrust_asl, .. } = part {
            thrust += thrust_asl;
        }
        if let Part::Engine{ thrust_asl, .. } = part {
            thrust += thrust_asl;
        }
    }

    (delta_v, thrust, altitude, velocity)
}

fn analyze_stages(stages: &Vec<Vec<Part>>) -> Vec<StageInfo> {
    let mut stage_info = Vec::new();
    let mut alt = 0.0;
    let mut vel = 0.0;
    for (i, stage) in stages.iter().enumerate() {
        let mut payload_mass = 0.0;
        for j in (i+1)..stages.len() {
            payload_mass += get_stage_mass_wet(&stages[j])
        }
        let rocket_mass = payload_mass +  get_stage_mass_wet(stage);
        let (deltav, thrust, a, v) = integrate_dv2(&stage, payload_mass, alt, vel);
        alt = a;
        vel = v;
        stage_info.push(StageInfo{
            wet_mass: get_stage_mass_wet(stage),
            dry_mass: get_stage_mass_dry(stage),
            delta_v: deltav,
            twr: thrust / (GRAVITY * rocket_mass),
            burnout_altitude: alt,
            burnout_velocity: vel,
        });
    }
    stage_info
}


fn permute_parts(base_parts: &[Part]) -> Vec<Part> {
    let mut parts = base_parts.to_vec();

    loop {
        let r = random::<f32>() % 1.0;
        if r > 0.9 {
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


fn check_validity(stages: &Vec<Vec<Part>>, stage_info: &[StageInfo]) -> bool {
    let total_mass: f32 = stage_info.iter().map(|s| s.wet_mass).sum();
    let contains_command_pod = stages[stages.len()-1].iter().any(|x| x.get_name() == "Mk1 Command Pod");
    let second_stage_twr = if stage_info.len() > 1 {
        stage_info[1].twr > 0.5
    } else {
        true
    };

    total_mass < 18.0 &&
    contains_command_pod &&
    stage_info[0].twr > 1.5 && second_stage_twr
}


fn optimize_rocket(starting_rocket: &[Part], iterations: usize) {
    let mut current_rocket: Vec<Part> = starting_rocket.to_vec();
    let stages = rocket_stages(&current_rocket);
    let stage_info = analyze_stages(&stages);

    print_stage_info(&stage_info);
    let mut current_deltav: f32 = stage_info.iter().map(|s| s.delta_v).sum();
    println!("INITIAL DELTA-V: {}m/s", current_deltav as i32);

    let mut i = 0;
    while i < iterations {
        let rocket_permutation = permute_parts(&current_rocket);
        let permutation_stages = rocket_stages(&rocket_permutation);
        let permutation_info = analyze_stages(&permutation_stages);
        let permutation_deltav: f32 = permutation_info.iter().map(|s| s.delta_v).sum();
        
        let stage_description: Vec<&str> = rocket_permutation.iter().map(|s| s.get_name()).collect();
        if permutation_deltav > current_deltav && check_validity(&permutation_stages, &permutation_info) {
            current_rocket = rocket_permutation;
            current_deltav = permutation_deltav;
            println!("i={i}, NEW STAGE: {}", stage_description.join(", "));
            print!("DELTA-V: {}m/s", permutation_deltav as i32);
            print!(" | TWR: {}", permutation_info[0].twr);
            if permutation_info.len() > 1 {
                print!(" | TWR (2. STAGE): {}", permutation_info[1].twr);
            }
            print!("\n\n");
        }
        i += 1;
    }

    let stages = rocket_stages(&current_rocket);
    let stage_info = analyze_stages(&stages);

    print_stage_info(&stage_info);
    let current_deltav: f32 = stage_info.iter().map(|s| s.delta_v).sum();
    println!("FINAL DELTA-V: {}m/s", current_deltav as i32);
}


#[bench]
fn benchmark(b: &mut test::Bencher) {
    b.iter(|| optimize_rocket(DEFAULT_ROCKET_1, 10000));
}


fn main() {
    let args = Args::parse();

    optimize_rocket(DEFAULT_ROCKET_1, args.count);
}
 