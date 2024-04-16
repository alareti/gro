pub struct Port<I> {
    on_recv: Box<dyn Fn(I)>,
}

impl<I> Port<I> {
    pub fn new(on_recv: Box<dyn Fn(I)>) -> Self {
        Self { on_recv }
    }

    pub fn drive(&self, input: I) {
        (self.on_recv)(input);
    }
}

// S is for self or state
pub trait Driver<S> {
    fn set_driver(&mut self, driver: Port<S>) -> Result<(), Port<S>>;
}

// I is for input
pub trait Receiver<I> {
    fn take_driver(&mut self) -> Result<Port<I>, ()>;
}

pub fn connect<T>(mut driver: impl Driver<T>, mut receiver: impl Receiver<T>) {
    if driver.set_driver(receiver.take_driver().unwrap()).is_err() {
        panic!("Couldn't set driver");
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // fn loopback() {
    //     let on_recv = |input| {
    //         assert_eq!(&1, input);
    //     };

    //     let mut driver = Port::new(Box::new(on_recv));

    //     let driver = driver.drive(&1).unwrap();
    //     let _ = driver.drive(&1).unwrap();
    // }
}
