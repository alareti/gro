type Receiver<T> = Box<dyn Fn(T)>;
pub struct Driver<T> {
    on_recv: Option<Receiver<T>>,
}

impl<T> Driver<T> {
    pub fn new() -> Self {
        Self { on_recv: None }
    }

    pub fn set_rx(&mut self, on_recv: Receiver<T>) {
        self.on_recv = Some(on_recv);
    }

    pub fn drive(mut self, input: T) -> Result<Self, ()> {
        if let Some(ref on_recv) = self.on_recv {
            on_recv(input);

            let on_recv = self.on_recv.take().unwrap();
            let mut driver = Self::new();
            driver.set_rx(on_recv);

            Ok(driver)
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback() {
        let on_recv = |input| {
            assert_eq!(&1, input);
        };

        let mut driver = Driver::new();
        driver.set_rx(Box::new(on_recv));

        let driver = driver.drive(&1).unwrap();
        let _ = driver.drive(&1).unwrap();
    }

    #[test]
    #[should_panic]
    fn no_rx() {
        let driver = Driver::<usize>::new();
        let _ = driver.drive(0xDEADBEEF).unwrap();
    }
}
