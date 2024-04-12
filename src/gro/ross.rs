use std::sync::Arc;

type Callback<T> = Arc<dyn Fn(T)>;

pub struct Sender<T> {
    on_recv: Callback<T>,
}

impl<T> Sender<T> {
    pub fn new(on_recv: Callback<T>) -> Self {
        Self { on_recv }
    }

    pub fn send(self, input: T) -> Self {
        (self.on_recv.as_ref())(input);

        Self::new(self.on_recv)
    }
}

pub struct Receiver<T> {
    on_send: Callback<T>,
}

impl<T> Receiver<T> {
    pub fn new(on_send: Callback<T>) -> Self {
        Self { on_send }
    }

    pub fn recv(self, input: T) -> Self {
        (self.on_send.as_ref())(input);

        Self::new(self.on_send)
    }
}
