use super::Generator;

pub struct IntoIter<'a, Yield, Return> {
    pub(super) generator: Generator<'a, Yield, (), Return>,
}

impl<'a, Yield, Return> Iterator for IntoIter<'a, Yield, Return> {
    type Item = Yield;

    fn next(&mut self) -> Option<Self::Item> {
        self.generator.resume(()).yielded()
    }
}
