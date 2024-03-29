/// Threadpool ripped from the rust book
/// https://doc.rust-lang.org/book/ch20-00-final-project-a-web-server.html
pub mod threadpool {
    use std::sync::{mpsc, Mutex, Arc};
    use std::thread;

    pub struct ThreadPool {
        workers: Vec<Worker>,
        sender: mpsc::Sender<Message>,
    }

    impl ThreadPool {
        pub fn new(size: usize) -> ThreadPool {
            let (sender, receiver) = mpsc::channel();
            let receiver = Arc::new(Mutex::new(receiver));

            let mut workers = Vec::with_capacity(size);
            for id in 0..size {
                workers.push(Worker::new(id, Arc::clone(&receiver)));
            }

            ThreadPool {
                workers,
                sender,
            }
        }

        pub fn execute<F>(&self, f: F)
            where F: FnOnce() + Send + 'static
        {
            let job = Box::new(f);
            self.sender.send(Message::NewJob(job)).unwrap();
        }
    }

    impl Drop for ThreadPool {
        fn drop(&mut self) {
            println!("Sending terminate message to all workers.");

            for _ in &mut self.workers {
                self.sender.send(Message::Terminate).unwrap();
            }

            println!("Shutting down all workers.");

            for worker in &mut self.workers {
                println!("Shutting down worker {}", worker.id);

                if let Some(thread) = worker.thread.take() {
                    thread.join().unwrap();
                }
            }
        }
    }

    trait FnBox {
        fn call_box(self: Box<Self>);
    }

    impl<F: FnOnce()> FnBox for F {
        fn call_box(self: Box<F>) {
            (*self)()
        }
    }

    type Job = Box<dyn FnBox + Send + 'static>;

    enum Message {
        NewJob(Job),
        Terminate,
    }

    struct Worker {
        id: usize,
        thread: Option<thread::JoinHandle<()>>,
    }

    impl Worker {
        fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
            let thread = thread::spawn(move || {
                loop {
                    match receiver.lock().unwrap().recv().unwrap() {
                        Message::NewJob(job) => {
//                        println!("Worker {} got a job; executing.", id);
                            job.call_box();
                        },
                        Message::Terminate => {
//                        println!("Worker {} was told to terminate.", id);
                            break;
                        },
                    }
                }
            });

            Worker {
                id,
                thread: Some(thread),
            }
        }
    }
}