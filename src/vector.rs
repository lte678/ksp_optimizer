use std::ops;

#[derive(Copy, Clone, Debug)]
pub struct Vector<const N: usize> {
    pub data: [f32; N],
}


impl<const N: usize> Vector<N> {
    pub fn abs(mut self) -> Vector<N> {
        for i in 0..N {
            self[i] = self[i].abs();
        }
        self
    }

    pub fn sum(self) -> f32 {
        self.data.iter().sum()
    }
}


fn _scal_mul<const N: usize>(mut v: Vector<N>, _rhs: f32) -> Vector<N> {
    // Scales vector v by scalar s.
    for i in 0..N {
        v[i] *= _rhs;
    }
    v
} 


impl<const N: usize> ops::Index<usize> for Vector<N> {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}


impl<const N: usize> ops::IndexMut<usize> for Vector<N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}


impl<const N: usize> ops::Add<Vector<N>> for Vector<N> {
    type Output = Vector<N>;

    fn add(mut self, _rhs: Vector<N>) -> Vector<N> {
        // Adds vectors v1 and v2.
        for i in 0..N {
            self[i] += _rhs[i];
        }
        self
    } 
}


impl<const N: usize> ops::Add<f32> for Vector<N> {
    type Output = Vector<N>;

    fn add(mut self, _rhs: f32) -> Vector<N> {
        // Adds scalar to vector.
        for i in 0..N {
            self[i] += _rhs;
        }
        self
    } 
}


impl<const N: usize> ops::Sub<Vector<N>> for Vector<N> {
    type Output = Vector<N>;

    fn sub(mut self, _rhs: Vector<N>) -> Vector<N> {
        // Subtracts vectors v1 and v2.
        for i in 0..N {
            self[i] -= _rhs[i];
        }
        self
    } 
}


impl<const N: usize> ops::Sub<f32> for Vector<N> {
    type Output = Vector<N>;

    fn sub(mut self, _rhs: f32) -> Vector<N> {
        // Subtracts scalar from vector.
        for i in 0..N {
            self[i] -= _rhs;
        }
        self
    } 
}


impl<const N: usize> ops::Mul<f32> for Vector<N> {
    type Output = Vector<N>;
    fn mul(self, _rhs: f32) -> Vector<N> { _scal_mul(self, _rhs) }
}


impl<const N: usize> ops::Mul<Vector<N>> for f32 {
    type Output = Vector<N>;
    fn mul(self, _rhs: Vector<N>) -> Vector<N> { _scal_mul(_rhs, self) }
}


impl<const N: usize> ops::Div<f32> for Vector<N> {
    type Output = Vector<N>;
    fn div(self, _rhs: f32) -> Vector<N> { _scal_mul(self, 1.0/_rhs) }
}


impl<const N: usize> ops::Mul<Vector<N>> for Vector<N> {
    type Output = Vector<N>;
    fn mul(mut self, _rhs: Vector<N>) -> Vector<N> {
        // Multiplies vectors element-wise.
        for i in 0..N {
            self[i] *= _rhs[i];
        }
        self
    }
}


impl<const N: usize> ops::Div<Vector<N>> for Vector<N> {
    type Output = Vector<N>;
    fn div(mut self, _rhs: Vector<N>) -> Vector<N> {
        // Multiplies vectors element-wise.
        for i in 0..N {
            self[i] /= _rhs[i];
        }
        self
    }
}
