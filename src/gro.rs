use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, RwLock,
};
use std::thread;

mod ross;

trait Reducer<S> {
    fn reduce(&self, state: &mut S);
}

trait Deducer<I> {
    fn deduce(&mut self, input: &I);
}

impl<I, S> Reducer<S> for I
where
    S: Deducer<I>,
{
    fn reduce(&self, state: &mut S) {
        state.deduce(self);
    }
}

impl<S, I> Deducer<I> for S
where
    I: Reducer<S>,
{
    fn deduce(&mut self, input: &I) {
        input.reduce(self);
    }
}

// Input is some type which can reduce self's state
type Input<'a, S> = &'a dyn Reducer<S>;
type Updater<S> = Arc<dyn Fn(Input<S>)>;

// Callback is something that expects self's state
// as input
type Callback<S> = Arc<dyn Fn(&S)>;
type Outputter<S> = Arc<dyn Fn(Callback<S>)>;

pub struct Gro<'a, S> {
    handle: Arc<thread::JoinHandle<()>>,
    sender: ross::Sender<Input<'a, S>>,
    receiver: ross::Receiver<Callback<S>>,
}

impl<'a, S> Gro<'a, S>
where
    S: 'static,
{
    fn new(init: S) -> Self {
        let state = Arc::new(RwLock::new(init));

        // Interpret as 'ready to transmit'
        let ready = Arc::new(AtomicBool::new(false));

        let handle = Arc::new(thread::spawn(move || {}));

        let c_state = Arc::clone(&state);
        let c_ready = Arc::clone(&ready);
        let c_handle = Arc::clone(&handle);

        // Assuming that updater is running in a thread other than
        // gro's own thread (i.e. wrapped in Sender)
        let updater: Updater<S> = Arc::new(move |input: Input<S>| {
            // Wait until gro is done outputting state to all consumers.
            // The consumers need to in turn process their own state
            // updates, so we park until they are done.
            // Outputter will set ready to false to indicate it
            // needs a state update to continue transmitting.
            while c_ready.load(Ordering::Relaxed) {
                thread::park();
            }

            // Perform state update
            input.reduce(&mut c_state.write().unwrap());

            // Indicate to outputter that state update
            // is complete
            c_ready.store(true, Ordering::Relaxed);
            c_handle.thread().unpark();
        });

        let c_state = Arc::clone(&state);
        let c_ready = Arc::clone(&ready);
        let c_handle = Arc::clone(&handle);

        // Assuming that outputter is running in a thread other than
        // gro's own thread (i.e. wrapped in Receiver)
        let outputter: Outputter<S> = Arc::new(move |callback| {
            while !c_ready.load(Ordering::Relaxed) {
                thread::park();
            }

            // Perform state update
            // input.reduce(&mut c_state.write().unwrap());
            callback(&c_state.read().unwrap());

            // Indicate to udpater that state update
            // is complete
            c_ready.store(false, Ordering::Relaxed);
            c_handle.thread().unpark();
        });

        let sender = ross::Sender::new(updater);
        let receiver = ross::Receiver::new(outputter);

        Gro {
            handle,
            sender,
            receiver,
        }
    }
}
