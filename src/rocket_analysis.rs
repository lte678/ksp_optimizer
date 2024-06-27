use crate::parts::{SOLID_FUEL_DENSITY, EFF_FUEL_DENSITY};
use crate::parts::{rocket_stages, part_mass_wet, part_fuel_mass, part_mass_dry};
use crate::parts::{Part, Stage};
use crate::parts::PartVariant::*;
use crate::vector::Vector;
use crate::integrator;
use crate::kerbin;


pub const GRAVITY: f32 = 9.81;


pub struct StageInfo {
    pub wet_mass: f32,
    pub dry_mass: f32,
    pub twr: f32,
    pub delta_v: f32,
    pub burnout_altitude: f32,
    pub burnout_velocity: f32,
}


pub struct RocketInfo {
    pub launch_mass: f32,
    pub delta_v: f32,
    pub part_count: usize,
    pub stage_info: Vec<StageInfo>,
    pub stages: Vec<Stage>,
    pub final_altitude: f32,
}


pub fn print_rocket_info(rocket_info: &RocketInfo) {
    for (i, stage) in rocket_info.stage_info.iter().enumerate() {
        print_stage_summary(stage, &format!("STAGE  {i}"));
    }
    print_rocket_summary(rocket_info);
}


pub fn print_stage_summary(stage_info: &StageInfo, header: &str) {
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


pub fn print_rocket_summary(rocket_info: &RocketInfo) {
    println!("============ ROCKET ============");
    println!("      LAUNCH MASS: {:.2}t", rocket_info.launch_mass);
    println!("          DELTA-V: {}m/s", rocket_info.delta_v as i32);
    println!("       PART COUNT: {}", rocket_info.part_count);
    println!("");
}


fn get_burnout_times(stage: &[Part]) -> Vec<f32> {
    let mut burnout_times = Vec::new();
    let fuel_mass = part_fuel_mass(stage) * EFF_FUEL_DENSITY;
    let mut liquid_mass_flow: f32 = 0.0;
    for part in stage {
        if let SolidBooster{ fuel, thrust_asl, isp_asl, ..} = part.variant {
            let solid_fuel = fuel * SOLID_FUEL_DENSITY;
            let solid_mass_flow = thrust_asl / (isp_asl * GRAVITY);
            if solid_fuel > 1e-6 && solid_mass_flow > 1e-6 {
                burnout_times.push(solid_fuel / solid_mass_flow);
            }
        }
        if let Engine{ thrust_asl, isp_asl, .. } = part.variant {
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

    let fuel_mass = part_fuel_mass(stage) * EFF_FUEL_DENSITY;

    let mut liquid_thrust_asl: f32 = 0.0;
    let mut liquid_thrust_vac: f32 = 0.0;
    let mut liquid_mass_flow: f32 = 0.0;
    let mut solid_rockets: Vec<(f32, f32, f32, f32)> = Vec::new();
    for part in stage {
        if let SolidBooster{ fuel, thrust_asl, thrust_vac, isp_asl, ..} = part.variant {
            solid_rockets.push((fuel * SOLID_FUEL_DENSITY, thrust_asl, thrust_vac, thrust_asl / (isp_asl * GRAVITY)));
        }
        if let Engine{ thrust_asl,  thrust_vac, isp_asl, .. } = part.variant {
            liquid_thrust_asl += thrust_asl;
            liquid_thrust_vac += thrust_vac;
            liquid_mass_flow += thrust_asl / (isp_asl * GRAVITY)
        }
    }

    let atmo_p = kerbin::get_pressure(altitude);
    let mut thrust = 0.0;
    let mut mass = payload_mass + part_mass_wet(stage);
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


fn integrate_dv(stage: &[Part], payload_mass: f32, altitude: f32, velocity: f32) -> (f32, f32, f32, f32) {
    let f = |t, state| flight_dynamics(t, &state, stage, payload_mass);

    let mut times = get_burnout_times(stage);
    times.insert(0, 0.0);

    let mut delta_v = 0.0;
    let mut velocity = velocity;
    let mut altitude = altitude;

    for t_i in 0..times.len()-1 {
        let (res, _) = integrator::rk45(
            &f, 
            Vector{ data: [delta_v, velocity, altitude] },
            times[t_i]+1e-6, times[t_i+1]-1e-3,
            Vector{ data: [1e-3, 1e-9, 1e-9]},
            1e-4
        );
        delta_v  = res[0];
        velocity = res[1];
        altitude = res[2];
    }

    // Get thrust information for TWR ratio calculation. This assumes nothing has burned out in the stage.
    let mut thrust = 0.0;
    for part in stage {
        thrust += match part.variant {
            SolidBooster{ thrust_asl, .. } => thrust_asl,
            Engine{ thrust_asl, .. } => thrust_asl,
            _ => 0.0
        }
    }

    (delta_v, thrust, altitude, velocity)
}

pub fn analyze_stages(stages: &Vec<Vec<Part>>) -> Vec<StageInfo> {
    let mut stage_info = Vec::new();
    let mut alt = 0.0;
    let mut vel = 0.0;
    for (i, stage) in stages.iter().enumerate() {
        let mut payload_mass = 0.0;
        for j in (i+1)..stages.len() {
            payload_mass += part_mass_wet(&stages[j])
        }
        let rocket_mass = payload_mass +  part_mass_wet(stage);
        let (deltav, thrust, a, v) = integrate_dv(&stage, payload_mass, alt, vel);
        alt = a;
        vel = v;
        stage_info.push(StageInfo{
            wet_mass: part_mass_wet(stage),
            dry_mass: part_mass_dry(stage),
            delta_v: deltav,
            twr: thrust / (GRAVITY * rocket_mass),
            burnout_altitude: alt,
            burnout_velocity: vel,
        });
    }
    stage_info
}


pub fn analyze_rocket(rocket: &Vec<Part>) -> RocketInfo {
    let stages = rocket_stages(rocket);
    let stage_info = analyze_stages(&stages);
    let launch_mass = part_mass_wet(&rocket);
    let delta_v=  stage_info.iter().map(|s| s.delta_v).sum();
    let part_count = rocket.len();
    let final_altitude = stage_info.last().unwrap().burnout_altitude;

    RocketInfo {
        launch_mass,
        delta_v,
        part_count,
        stage_info,
        stages,
        final_altitude,
    }
}