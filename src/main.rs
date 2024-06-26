mod vector;
mod kerbin;
mod integrator;

use core::fmt;
use std::fmt::Debug;

use crate::vector::Vector;
use rand::prelude::*;


#[derive(Debug, Copy, Clone)]
enum Part {
    SolidBooster {
        name: &'static str,
        mass: f32,
        thrust_asl: f32,
        thrust_vac: f32,
        isp_asl: f32,
        isp_vac: f32,
        fuel: f32,
    },
    Engine {
        name: &'static str,
        mass: f32,
        thrust_asl: f32,
        thrust_vac: f32,
        isp_asl: f32,
        isp_vac: f32,
    },
    Tank {
        name: &'static str,
        mass: f32,
        fuel: f32,
    },
    Decoupler {
        name: &'static str,
        mass: f32,
    },
    Structure {
        name: &'static str,
        mass: f32,
    }
}


impl Part {
    fn get_name(&self) -> &'static str {
        match self {
            Part::SolidBooster { name, .. } => name,
            Part::Engine { name, .. } => name,
            Part::Tank { name, ..} => name,
            Part::Decoupler { name, .. } => name,
            Part::Structure { name, .. } => name,
        }
    }
}


impl fmt::Display for Part {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_name())
    }
}

const PART_TD12: Part = Part::Decoupler {
    name: "TD-12", mass: 0.04 };
const PART_RT5: Part = Part::SolidBooster {
    name: "RT-5",
    mass: 0.45, fuel: 140.0,
    thrust_asl: 162.91, thrust_vac: 192.0,
    isp_asl: 140.0, isp_vac: 165.0 };
const PART_RT10: Part = Part::SolidBooster {
    name: "RT-10",
    mass: 0.75, fuel: 375.0,
    thrust_asl: 197.90, thrust_vac: 227.0,
    isp_asl: 170.0, isp_vac: 195.0 };
const PART_BACC: Part = Part::SolidBooster {
    name: "BACC",
    mass: 1.5, fuel: 820.0,
    thrust_asl: 250.0, thrust_vac: 300.0,
    isp_asl: 175.0, isp_vac: 210.0 };
const PART_LVT30: Part = Part::Engine {
    name: "LV-T30",
    mass: 1.25,
    thrust_asl: 205.16, thrust_vac: 240.0,
    isp_asl: 265.0, isp_vac: 310.0 };
const PART_LVT45: Part = Part::Engine {
    name: "LV-T45",
    mass: 1.50,
    thrust_asl: 167.97, thrust_vac: 215.0,
    isp_asl: 250.0, isp_vac: 320.0 };
const PART_FLT100: Part = Part::Tank {
    name: "FL-T100", mass: 0.0625, fuel: 45.0 };
const PART_FLT200: Part = Part::Tank {
    name: "FL-T200", mass: 0.125, fuel: 90.0 };
const PART_FLT400: Part = Part::Tank {
    name: "FL-T400", mass: 0.25, fuel: 180.0 };
const PART_MK1_POD: Part = Part::Structure {
    name: "Mk1 Command Pod", mass: 0.84 };

const PART_CATALOGUE: &[Part] = &[
    PART_TD12,
    PART_RT5,
    PART_RT10,
    PART_BACC,
    PART_LVT30,
    PART_LVT45,
    PART_FLT100,
    PART_FLT200,
    PART_FLT400,
    PART_MK1_POD,
];


const SOLID_FUEL_DENSITY: f32 = 0.0075;
const EFF_FUEL_DENSITY: f32 = 0.005 * 20.0 / 9.0;
const GRAVITY: f32 = 9.81;


struct StageInfo {
    wet_mass: f32,
    dry_mass: f32,
    twr: f32,
    delta_v: f32,
    burnout_altitude: f32,
    burnout_velocity: f32,
}


fn get_part_fuel(parts: &[Part]) -> f32 {
    let mut sum: f32 = 0.0;
    for part in parts {
        sum += match part {
            Part::Tank { fuel, .. } => fuel,
            _ => &0.0,
        }
    }
    sum
}


fn get_part_solid_fuel(parts: &[Part]) -> f32 {
    let mut sum: f32 = 0.0;
    for part in parts {
        sum += match part {
            Part::SolidBooster { fuel, .. } => fuel,
            _ => &0.0,
        }
    }
    sum
}


fn get_stage_mass_dry(parts: &[Part]) -> f32 {
    let mut sum: f32 = 0.0;
    for part in parts {
        sum += match part {
            Part::Decoupler {mass, ..} => mass,
            Part::Engine {mass, ..} => mass,
            Part::SolidBooster { mass, .. } => mass,
            Part::Tank {mass, ..} => mass,
            Part::Structure { mass, .. } => mass,
        }
    }
    sum
}


fn get_stage_mass_wet(parts: &[Part]) -> f32 {
    let part_mass = get_stage_mass_dry(parts);
    let fuel_mass = get_part_fuel(parts) * EFF_FUEL_DENSITY;
    let solid_fuel_mass = get_part_solid_fuel(parts) * SOLID_FUEL_DENSITY;
    part_mass + fuel_mass + solid_fuel_mass
}


fn rocket_stages(parts: &[Part]) -> Vec<Vec<Part>> {
    // Splits a rocket into stages, as separated by decouplers
    // Decouplers are included in the lower stage (because they are jettisoned with them)
    let mut stages = Vec::new();
    let mut stage_parts = Vec::<Part>::new();
    for part in parts {
        stage_parts.push(*part);
        match part {
            Part::Decoupler {..} => { stages.push(stage_parts); stage_parts = Vec::new(); }
            _ => {},
        }
    }
    stages.push(stage_parts);
    stages
}


fn print_stage_info(stages: &Vec<StageInfo>) {
    for (i, stage) in stages.iter().enumerate() {
        print_summary(stage, &format!("STAGE  {i}"));
    }
}


fn print_summary(stage_info: &StageInfo, header: &str) {
    println!("=========== {header:8} ==========");
    println!("        PART MASS: {:.2}t", stage_info.dry_mass);
    println!("        FUEL MASS: {:.2}t", stage_info.wet_mass - stage_info.dry_mass);
    println!("         WET MASS: {:.2}t", stage_info.wet_mass);
    println!("          DELTA-V: {}m/s", stage_info.delta_v as i32);
    println!(" THRUST TO WEIGHT: {:.2}", stage_info.twr);
    println!(" BURNOUT ALTITUDE: {}km", (stage_info.burnout_altitude / 1000.0) as i32);
    println!(" BURNOUT VELOCITY: {}m/s", stage_info.burnout_velocity as i32);
    println!("");
}


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
        if (random::<f32>() % 1.0) > 0.50 {
            let part_type = random::<usize>() % PART_CATALOGUE.len(); 
            let part_i = random::<usize>() % (parts.len() + 1); 
            parts.insert(part_i, PART_CATALOGUE[part_type]);
        } else {
            if parts.len() > 0 {
                let part_i = random::<usize>() % parts.len(); 
                parts.remove(part_i);
            }
        }
        // Allow multiple permutations to happen at once.
        if (random::<f32>() % 1.0) > 0.6 {
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


fn main() {
    let mut current_rocket: Vec<Part> = [
        PART_RT10,
        PART_TD12,
        PART_LVT45,
        PART_FLT100,
        PART_MK1_POD
    ].to_vec();

    let stages = rocket_stages(&current_rocket);
    let stage_info = analyze_stages(&stages);

    print_stage_info(&stage_info);
    let mut current_deltav: f32 = stage_info.iter().map(|s| s.delta_v).sum();
    println!("INITIAL DELTA-V: {}m/s", current_deltav as i32);

    let mut i = 0;
    while i < 50000 {
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
 