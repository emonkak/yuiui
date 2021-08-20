pub trait DrawPipeline: Default {
    fn compose(&mut self, other: Self);
}
