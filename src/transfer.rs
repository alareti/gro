use crate::comms;

trait Reducer<R> {
    fn reduce(&self, reducee: &mut R);
}

struct Transfer<T> {
    data: T,
}
