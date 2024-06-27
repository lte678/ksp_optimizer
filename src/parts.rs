use core::fmt;

pub const SOLID_FUEL_DENSITY: f32 = 0.0075;
pub const EFF_FUEL_DENSITY: f32 = 0.005 * 20.0 / 9.0;


#[derive(Debug, Copy, Clone)]
pub enum PartVariant {
    SolidBooster {
        thrust_asl: f32,
        thrust_vac: f32,
        isp_asl: f32,
        #[allow(unused)] // We want isp_vac in the database, but don't use it currently.
        isp_vac: f32,
        fuel: f32,
    },
    Engine {
        thrust_asl: f32,
        thrust_vac: f32,
        isp_asl: f32,
        #[allow(unused)]
        isp_vac: f32,
    },
    Tank { 
        fuel: f32,
    },
    Decoupler,
    Parachute,
    CommandPod,
}
use PartVariant::*;


#[derive(Debug, Copy, Clone)]
pub struct Part {
    pub name: &'static str,
    pub mass: f32,
    pub variant: PartVariant,
}


pub type Stage = Vec<Part>;


impl Part {
    const fn new(name: &'static str, mass: f32, variant: PartVariant) -> Part {
        Part {name, mass, variant}
    }
}


impl fmt::Display for Part {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}


pub fn part_fuel_mass(parts: &[Part]) -> f32 {
    parts.iter().map(|part| match part.variant {
        Tank { fuel, .. } => fuel,
        _ => 0.0,
    }).sum()
}


pub fn part_solid_fuel_mass(parts: &[Part]) -> f32 {
    parts.iter().map(|part| match part.variant {
        SolidBooster { fuel, .. } => fuel,
        _ => 0.0,
    }).sum()
}


pub fn part_mass_dry(parts: &[Part]) -> f32 {
    parts.iter().map(|part| part.mass).sum()
}


pub fn part_mass_wet(parts: &[Part]) -> f32 {
    let part_mass = part_mass_dry(parts);
    let fuel_mass = part_fuel_mass(parts) * EFF_FUEL_DENSITY;
    let solid_fuel_mass = part_solid_fuel_mass(parts) * SOLID_FUEL_DENSITY;
    part_mass + fuel_mass + solid_fuel_mass
}


pub fn rocket_stages(parts: &[Part]) -> Vec<Stage> {
    // Splits a rocket into stages, as separated by decouplers
    // Decouplers are included in the lower stage (because they are jettisoned with them)
    let mut stages = Vec::new();
    let mut stage_parts = Vec::<Part>::new();
    for part in parts {
        stage_parts.push(*part);
        match part.variant {
            PartVariant::Decoupler => { stages.push(stage_parts); stage_parts = Vec::new(); }
            _ => {},
        }
    }
    stages.push(stage_parts);
    stages
}


pub fn sort_stage(parts: &[Part]) -> Vec<Part> {
    let mut sorted: Vec<Part> = Vec::new();
    sorted.extend(parts.iter().filter(|p| matches!(p.variant, SolidBooster{..})));
    sorted.extend(parts.iter().filter(|p| matches!(p.variant, Engine{..})));
    sorted.extend(parts.iter().filter(|p| matches!(p.variant, Tank{..})));
    sorted.extend(parts.iter().filter(|p| matches!(p.variant, CommandPod)));
    sorted.extend(parts.iter().filter(|p| matches!(p.variant, Parachute)));
    sorted.extend(parts.iter().filter(|p| matches!(p.variant, Decoupler)));
    assert!(sorted.len() == parts.len());
    sorted
}


pub fn sort_rocket(rocket: &[Stage]) -> Vec<Stage> {
    let mut sorted: Vec<Stage> = Vec::new();
    for stage in rocket {
        sorted.push(sort_stage(stage));
    }
    sorted
}


pub fn print_rocket(stages: &[Stage]) {
    let mut it = stages.iter().peekable();
    while let Some(stage) = it.next() {
        for part in stage {
            print!("{} ", part.name);
        }
        if it.peek().is_some() {
            print!("// ");
        }
    }
}


const PART_TD12: Part = Part::new("TD-12", 0.04, Decoupler);
const PART_RT5: Part = Part::new("RT-5", 0.45, SolidBooster 
    { fuel: 140.0, thrust_asl: 162.91, thrust_vac: 192.0, isp_asl: 140.0, isp_vac: 165.0 });
const PART_RT10: Part = Part::new("RT-10", 0.75, SolidBooster
    { fuel: 375.0, thrust_asl: 197.90, thrust_vac: 227.0, isp_asl: 170.0, isp_vac: 195.0 });
const PART_BACC: Part = Part::new("BACC", 1.5, SolidBooster
    { fuel: 820.0, thrust_asl: 250.0, thrust_vac: 300.0, isp_asl: 175.0, isp_vac: 210.0 });
const PART_LVT30: Part = Part::new("LV-T30", 1.25, Engine
    {thrust_asl: 205.16, thrust_vac: 240.0, isp_asl: 265.0, isp_vac: 310.0 });
const PART_LVT45: Part = Part::new("LV-T45", 1.50, Engine
    {thrust_asl: 167.97, thrust_vac: 215.0, isp_asl: 250.0, isp_vac: 320.0 });
const PART_LV909: Part = Part::new("LV-909", 0.50, Engine
    {thrust_asl: 14.78, thrust_vac: 60.0, isp_asl: 85.0, isp_vac: 345.0 });
const PART_FLT100: Part = Part::new("FL-T100", 0.0625, Tank{ fuel: 45.0 });
const PART_FLT200: Part = Part::new("FL-T200", 0.125, Tank{ fuel: 90.0 });
const PART_FLT400: Part = Part::new("FL-T400", 0.25, Tank{ fuel: 180.0 });
const PART_FLT800: Part = Part::new("FL-T800", 0.50, Tank{ fuel: 360.0 });
const PART_MK1_POD: Part = Part::new("Mk1 Command Pod", 0.84, CommandPod);
const PART_MK16_CHUTE: Part = Part::new("Mk16 Parachute", 0.1, Parachute);


pub const PART_CATALOGUE: &[Part] = &[
    PART_TD12,
    PART_RT5,
    PART_RT10,
    PART_BACC,
    PART_LVT30,
    PART_LVT45,
    PART_LV909,
    PART_FLT100,
    PART_FLT200,
    PART_FLT400,
    PART_FLT800,
    PART_MK1_POD,
    PART_MK16_CHUTE,
];


pub const DEFAULT_ROCKET_1: &[Part] = &[
    PART_RT10,
    PART_TD12,
    PART_LVT45,
    PART_FLT100,
    PART_MK1_POD,
    PART_MK16_CHUTE,
];