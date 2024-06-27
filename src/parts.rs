use core::fmt;

pub const SOLID_FUEL_DENSITY: f32 = 0.0075;
pub const EFF_FUEL_DENSITY: f32 = 0.005 * 20.0 / 9.0;


#[derive(Debug, Copy, Clone)]
pub enum Part {
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


pub struct StageInfo {
    pub wet_mass: f32,
    pub dry_mass: f32,
    pub twr: f32,
    pub delta_v: f32,
    pub burnout_altitude: f32,
    pub burnout_velocity: f32,
}


impl Part {
    pub fn get_name(&self) -> &'static str {
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


pub fn get_part_fuel(parts: &[Part]) -> f32 {
    let mut sum: f32 = 0.0;
    for part in parts {
        sum += match part {
            Part::Tank { fuel, .. } => fuel,
            _ => &0.0,
        }
    }
    sum
}


pub fn get_part_solid_fuel(parts: &[Part]) -> f32 {
    let mut sum: f32 = 0.0;
    for part in parts {
        sum += match part {
            Part::SolidBooster { fuel, .. } => fuel,
            _ => &0.0,
        }
    }
    sum
}


pub fn get_stage_mass_dry(parts: &[Part]) -> f32 {
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


pub fn get_stage_mass_wet(parts: &[Part]) -> f32 {
    let part_mass = get_stage_mass_dry(parts);
    let fuel_mass = get_part_fuel(parts) * EFF_FUEL_DENSITY;
    let solid_fuel_mass = get_part_solid_fuel(parts) * SOLID_FUEL_DENSITY;
    part_mass + fuel_mass + solid_fuel_mass
}


pub fn rocket_stages(parts: &[Part]) -> Vec<Vec<Part>> {
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


pub fn print_stage_info(stages: &Vec<StageInfo>) {
    for (i, stage) in stages.iter().enumerate() {
        print_summary(stage, &format!("STAGE  {i}"));
    }
}


pub fn print_summary(stage_info: &StageInfo, header: &str) {
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

pub const PART_CATALOGUE: &[Part] = &[
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


pub const DEFAULT_ROCKET_1: &[Part] = &[
    PART_RT10,
    PART_TD12,
    PART_LVT45,
    PART_FLT100,
    PART_MK1_POD
];