use crate::sync::Driver;

use std::rc::Rc;

trait Reducer<S> {
    fn reduce(&self, state: &mut S);
}

struct Register<S> {
    state: Rc<S>,
}

type Input<S> = Box<dyn Reducer<S>>;

struct Builder<S> {
    init: S,
    input: Option<Rc<Driver<Input<S>>>>,
    output: Option<Rc<Driver<Box<S>>>>,
}

impl<S> Builder<S> {
    fn new(init: S) -> Self {
        Builder {
            init,
            input: None,
            output: None,
        }
    }

    // This will define an external driver's on_recv field
    fn input(mut self, driver: Rc<Driver<Input<S>>>) -> Self {
        self.input = Some(driver);
        self
    }

    // This will come to own the driver as its own
    // Boxed for symmetry with input
    fn output(mut self, driver: Rc<Driver<Box<S>>>) -> Self {
        self.output = Some(driver);
        self
    }
}
