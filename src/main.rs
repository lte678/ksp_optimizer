use std::iter::zip;


#[derive(Copy, Clone)]
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


fn print_stages(stages: &Vec<Vec<Part>>) {
    for (i, stage) in stages.iter().enumerate() {
        print_summary(stage, &format!("STAGE  {i}"));
    }
}


fn print_summary(parts: &[Part], header: &str) {
    let part_mass = get_stage_mass_dry(parts);
    let fuel_mass = get_part_fuel(parts) * EFF_FUEL_DENSITY;
    let solid_fuel_mass = get_part_solid_fuel(parts) * SOLID_FUEL_DENSITY;
    let total_mass = part_mass + fuel_mass + solid_fuel_mass;
    println!("=========== {header:8} ==========");
    println!("          PART MASS: {part_mass:.2}t");
    println!("          FUEL MASS: {fuel_mass:.2}t");
    println!("  FUEL MASS (SOLID): {solid_fuel_mass:.2}t");
    println!("         TOTAL MASS: {total_mass:.2}t");
    println!("");
}


fn integrate_dv(stage: &[Part], payload_mass: f32) -> f32 {
    const DT: f32 = 1.0;

    let mut fuel = get_part_fuel(stage);
    let mut mass = payload_mass + get_stage_mass_wet(stage);

    let mut liquid_thrust: f32 = 0.0;
    let mut liquid_mass_flow: f32 = 0.0;
    let mut solid_rockets: Vec<(f32, f32, f32)> = Vec::new();
    for part in stage {
        if let Part::SolidBooster{ fuel, thrust_asl, isp_asl, ..} = part {
            solid_rockets.push((*fuel, *thrust_asl, thrust_asl / (isp_asl * GRAVITY)));
        }
        if let Part::Engine{ thrust_asl, isp_asl, .. } = part {
            liquid_thrust += thrust_asl;
            liquid_mass_flow += thrust_asl / (isp_asl * GRAVITY)
        }
    }

    // Integrate the force over time
    let mut delta_v = 0.0; 
    let mut burning = true;
    while(burning) {
        burning = false;
        let mut thrust = 0.0;
        if fuel > 0.0 {
            burning = true;
            thrust += liquid_thrust;
            mass -= liquid_mass_flow * DT;
            fuel -= (liquid_mass_flow / EFF_FUEL_DENSITY) * DT;
        }
        for (s_fuel, s_thrust, s_mass_flow) in &mut solid_rockets {
            if *s_fuel > 0.0 {
                burning = true;
                thrust += *s_thrust;
                mass -= *s_mass_flow * DT;
                *s_fuel -= (*s_mass_flow / SOLID_FUEL_DENSITY) * DT;
            }
        }

        delta_v += DT * thrust / mass;
    }
    delta_v
}


fn main() {
    let test_rocket = [
        PART_RT10,
        PART_TD12,
        PART_LVT45,
        PART_FLT100,
        PART_MK1_POD
    ];

    let stages = rocket_stages(&test_rocket);
    print_stages(&stages);
    print_summary(&test_rocket, " ROCKET ");

    let mut total_deltav = 0.0;
    for (i, stage) in stages.iter().enumerate() {
        let mut payload_mass = 0.0;
        for j in (i+1)..stages.len() {
            payload_mass += get_stage_mass_wet(&stages[j])
        }
        let deltav = integrate_dv(&stage, payload_mass);
        total_deltav += deltav;
        println!("STAGE {i} DELTA-V: {}m/s", deltav as i32);
    }
    println!("TOTAL DELTA-V: {}m/s", total_deltav as i32);
}
