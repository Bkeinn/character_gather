// use std::thread;

// pub struct ThreadPool {
//     workers: Vec<Worker>,
// }

// struct Worker {
//     id: usize,
//     thread: thread::JoinHandle<()>,
// }

// impl ThreadPool {
//     pub fn new(size: usize) -> ThreadPool {
//         let mut workers = Vec::with_capacity(size);

//         for id in 0..size {
//             workers.push(Worker::new(id));
//         }

//         ThreadPool { workers }
//     }
//     pub fn spawn<F, T>(f: F) -> thread::JoinHandle<T>
//     where
//         F: FnOnce() -> T,
//         F: Send + 'static,
//         T: Send + 'static,
//     {
//     }
//     pub fn execute<F>(&self, f: F)
//     where
//         F: FnOnce() + Send + 'static,
//     {
//     }
// }

// impl Worker {
//     fn new(id: usize) -> Worker {
//         let thread = thread::spawn(|| {});
//         Worker { id, thread }
//     }
// }
