use std::thread;

struct Groat {
    handle: thread::JoinHandle<()>,
}

impl Groat {
    fn join(self) -> thread::Result<()> {
        self.handle.join()
    }
}

struct Builder<T> {
    initial_state: T,
    name: Option<String>,
}

impl<T> Builder<T>
where
    T: Send + 'static + std::fmt::Debug,
{
    fn new(initial_state: T) -> Self {
        Builder {
            initial_state,
            name: None,
        }
    }

    fn name(self, name: String) -> Self {
        Builder {
            initial_state: self.initial_state,
            name: Some(name),
        }
    }

    fn spawn(self) -> Groat {
        let thread_builder = thread::Builder::new();

        let thread_builder = match self.name {
            Some(name) => thread_builder.name(name),
            None => thread_builder,
        };

        let handle = thread_builder
            .spawn(move || {
                let groat_name = match thread::current().name() {
                    Some(name) => String::from(name),
                    None => format!("{:?}", thread::current().id()),
                };
                println!("Spawned new groat named {groat_name}");

                let state = self.initial_state;
                println!("{groat_name}\tInitial state: {state:#?}");
            })
            .unwrap();

        Groat { handle }
    }
}

trait Reducer {
    type Input;

    fn reducer(&mut self, input: &Self::Input);
}

#[derive(Debug)]
struct StateA(String);
impl Reducer for StateA {
    type Input = StateB;

    fn reducer(&mut self, input: &Self::Input) {
        self.0.push_str(&input.0);
    }
}

#[derive(Debug)]
struct StateB(String);
impl Reducer for StateB {
    type Input = StateA;

    fn reducer(&mut self, input: &Self::Input) {
        self.0.push_str(&input.0);
    }
}

fn main() {
    let groat_builder = Builder::new(StateA(String::from(""))).name(String::from("GroatA"));
    let groat = groat_builder.spawn();
    let _ = groat.join();
}
