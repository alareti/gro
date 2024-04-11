use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, RwLock,
};
use std::thread;

pub trait Reducer {
    type State;

    fn reduce(&self, state: &mut Self::State);
}

type Updater<S> = Arc<dyn Fn(Arc<dyn Reducer<State = S>>) + Send + Sync>;

pub struct Gro<S> {
    handle: Arc<thread::JoinHandle<()>>,
    updater: Updater<S>,
    subscribers: Vec<Updater<S>>,
}

impl<S> Gro<S>
where
    S: Send + Sync + 'static,
{
    pub fn new(init: S) -> Self {
        let state = Arc::new(RwLock::new(init));

        // Interpret as 'ready to transmit'
        let ready = Arc::new(AtomicBool::new(false));

        let c_state = Arc::clone(&state);
        let c_ready = Arc::clone(&ready);

        // Assuming that updater is running in a thread other than
        // gro's own thread.
        let updater: Updater<S> = Arc::new(move |input| {
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
        });

        let handle = Arc::new(thread::spawn(move || {}));

        // Assuming that outputter is running in the gro's own
        // thread.
        // let outputter = ();

        Gro {
            handle,
            updater,
            subscribers: vec![],
        }
    }

    pub fn send(&self, input: Arc<dyn Reducer<State = S>>) {
        (self.updater)(input);
    }

    pub fn subscribe(&self, on_update: Arc<dyn Fn(&S) + Send + Sync>) {}
}
