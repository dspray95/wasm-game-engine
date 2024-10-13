use super::strider::MatrixShape;

pub struct Matrix{
    buffer: Vec<f32>,
    shape: MatrixShape
}

impl Matrix {

    //Unary operations - no need to modify the buffer
    pub fn exp(&self) -> Matrix{
        let mut result = vec![0.0; self.buffer.len()];
        for i in 0..self.buffer.len() {
            result[i] = self.buffer[i].exp();
        }
        Matrix{
            buffer: result,
            shape: self.shape.clone()
        }
    }

    pub fn log(&self, base: f32) -> Matrix{
        let mut result = vec![0.0; self.buffer.len()];
        for i in 0..self.buffer.len() {
            result[i] = self.buffer[i].log(base);
        }
        Matrix{
            buffer: result,
            shape: self.shape.clone()
        }
    }
}
pub fn matrix_2x2(matrix: [[f32; 2]; 2]) -> Matrix {
    let mut buffer = vec!(0.0; 4);
    buffer[..2].copy_from_slice(&matrix[0]);
    buffer[2..].copy_from_slice(&matrix[1]);
    let shape = vec![2, 2];
    let matrix_shape = MatrixShape::new(shape);
    return Matrix {
        buffer,
        shape: matrix_shape
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_2x2_identity() {
        let matrix = [[1.0, 0.0], [0.0, 1.0]];
        let result = matrix_2x2(matrix);
        assert_eq!(result.buffer, vec![1.0, 0.0, 0.0, 1.0]);
        assert_eq!(result.shape.dimensions(), &[2, 2]);
    }

    #[test]
    fn test_matrix_2x2_zeros() {
        let matrix = [[0.0, 0.0], [0.0, 0.0]];
        let result = matrix_2x2(matrix);
        assert_eq!(result.buffer, vec![0.0, 0.0, 0.0, 0.0]);
        assert_eq!(result.shape.dimensions(), &[2, 2]);
    }

    #[test]
    fn test_matrix_2x2_arbitrary() {
        let matrix = [[1.0, 2.0], [3.0, 4.0]];
        let result = matrix_2x2(matrix);
        assert_eq!(result.buffer, vec![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(result.shape.dimensions(), &[2, 2]);
    }
}
