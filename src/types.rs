#[derive(Debug, Clone)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vec2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

pub type Dimensions = Vec2<usize>;
pub type Position = Vec2<usize>;

pub struct Matrix2<T: Default + Clone> {
    dimensions: Dimensions,
    values: Vec<T>,
}

impl<T: Default + Clone> Matrix2<T> {
    pub fn new(dimensions: &Dimensions) -> Self {
        Self {
            dimensions: dimensions.clone(),
            values: vec![T::default(); dimensions.x * dimensions.y],
        }
    }

    pub fn get(&self, x: usize, y: usize) -> &T {
        &self.values[y * self.dimensions.x + x]
    }

    pub fn set(&mut self, x: usize, y: usize, value: T) {
        self.values[y * self.dimensions.x + x] = value;
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.values.iter()
    }

    pub fn clear(&mut self) {
        for value in &mut self.values {
            *value = T::default();
        }
    }
}
