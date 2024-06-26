use std::f32::EPSILON;
use crate::vector::Vector;


#[allow(dead_code)]
pub struct IntegratorSolInfo {
    pub total_steps: usize,
}


pub fn rk45<const N: usize, F>(f: F, y0: Vector<N>, t0: f32, tend: f32, atol: Vector<N>, rtol: f32) -> (Vector<N>, IntegratorSolInfo)
where
    F: Fn(f32, Vector<N>) -> Vector<N>
{
    // That this integrator uses half-precision is unusual, but should double performance, since we will not
    // even be running close to that precision range.
    // Sources:
    // [1] https://maths.cnam.fr/IMG/pdf/RungeKuttaFehlbergProof.pdf

    let mut y: Vector<N> = y0;
    let mut t = t0;
    let mut h: f32 = 10.0;
    let mut step_count: usize = 0;
    while t < tend {
        if t + h > tend {
            h = tend - t;
        }

        // Evaluate function 6 times. This is sufficient for both the 4. and 5. order integrator to use, as they are embedded.
        let k1 = h * f(t              , y);
        let k2 = h * f(t + h/4.0      , y + k1/4.0);
        let k3 = h * f(t + h*3.0/8.0  , y + k1*3.0/32.0      + k2*9.0/32.0);
        let k4 = h * f(t + h*12.0/13.0, y + k1*1932.0/2197.0 - k2*7200.0/2197.0 + k3*7296.0/2197.0);
        let k5 = h * f(t + h          , y + k1*439.0/216.0   - k2*8.0           + k3*3680.0/513.0  - k4*845.0/4104.0);
        let k6 = h * f(t + h/2.0      , y - k1*8.0/27.0      + k2*2.0           - k3*3544.0/2565.0 + k4*1859.0/4104.0 - k5*11.0/40.0);
        // Apply RK4
        let y1 = y + (25.0/216.0) * k1 + (1408.0/2565.0) * k3 + (2197.0/4104.0) * k4 - (1.0/5.0) * k5;
        // Apply RK5
        let y2 = y + (16.0/135.0) * k1 + (6656.0/12825.0) * k3 + (28561.0/56430.0) * k4 - (9.0/50.0) * k5 + (2.0/55.0) * k6;

        let error =  ((y2 - y1).abs() + EPSILON) / (atol + rtol*y2.abs());
        let error = (error * error).sum().powf(0.5);
        // println!("error: {}, f({}): {:?}, y: {:?}", error, t+h, f(t+h, y), y2);
        if error <= 1.0 {
            t += h;
            y = y1;
        }
        // Optimize step size to achieve desired accuracy
        h *= 0.84 * error.powf(-0.25);

        step_count += 1;
    }

    (y, IntegratorSolInfo{ total_steps: step_count })
}
