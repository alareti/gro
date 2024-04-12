type Receiver<T> = Box<dyn Fn(T)>;

pub struct Driver<T> {
    on_recv: Option<Receiver<T>>,
}

impl<T> Driver<T> {
    pub fn new(on_recv: Option<Receiver<T>>) -> Self {
        Self { on_recv }
    }

    pub fn drive(self, input: T) -> Result<(), T> {
        match self.on_recv.as_ref() {
            Some(rx) => {
                rx(input);
                Ok(())
            }
            None => Err(input),
        }
    }

    pub fn set_rx(&mut self, on_recv: Receiver<T>) -> Result<(), Receiver<T>> {
        match self.on_recv {
            Some(_) => Err(on_recv),
            None => {
                self.on_recv = Some(on_recv);
                Ok(())
            }
        }
    }
}
