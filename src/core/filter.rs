pub trait Filter {
    fn radius(&self) -> i32;

    fn weight(&self, x: f32, y: f32) -> f32;
}
