#![allow(dead_code)]
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

mod test;

///
/// A struct that performs a lengthy task in a seperate thread with a queue
/// Types:
/// 
/// Input: 'static
///     Input type
///     Must be statically specified (ie. BackgroundWorker<i32, f32>)
///     Must derive: std::marker::Send      --To send between threads
/// 
/// Output: 'static
///     Output type
///     Must be statically specified (ie. BackgroundWorker<i32, f32>)
///     Must derive: std::marker::Send,     --To send between threads
///                  std::clone::Clone,     --To be able to clone in the pop_range function
///                  std::cmp::PartialEq
/// 

#[derive(Debug)]
struct BackgroundWorker<Input: 'static, Output: 'static>
where
    Input: std::marker::Send,
    Output: std::marker::Send + std::clone::Clone + std::cmp::PartialEq,
{
    ///
    /// A user-provided function that takes in input and produces output
    /// 
    function: fn(Input) -> Output,
    ///
    /// A worker-created JoinHandle that starts when something is queued
    /// 
    thread_handle: Option<JoinHandle<()>>,
    ///
    /// outqueue/inqueue:
    ///     A public VecDequeue<Output> which is wrapped in a Mutex and an Arc
    ///     Only made public to allow for more precise use of the queues, not
    ///     reccomended to be used in any general circumstance, use the pop and
    ///     push and their respective _vec variants for putting stuff in and out
    /// 
    pub outqueue: Arc<Mutex<VecDeque<Output>>>,
    pub inqueue: Arc<Mutex<VecDeque<Input>>>,
    ///
    /// An atomic boolean that is set to true when new() is called or when a
    /// thread ends, otherwise false
    /// 
    thread_dead: Arc<AtomicBool>,
}

impl<Input: 'static, Output: 'static> BackgroundWorker<Input, Output>
where
    Input: std::marker::Send,
    Output: std::marker::Send + std::clone::Clone + std::cmp::PartialEq,
{
    ///
    /// Creates a BackgroundWorker with the specified function
    /// Parameters:
    ///     func:
    ///         A function pointer (can also be a closure) which takes in an input
    ///         and produces an output. Write this function assuming it will run 
    ///         on a seperate thread rather than the main thread. 
    /// 
    pub fn new(func: fn(Input) -> Output) -> BackgroundWorker<Input, Output> {
        BackgroundWorker {
            function: func,
            outqueue: Arc::new(Mutex::new(VecDeque::new())),
            inqueue: Arc::new(Mutex::new(VecDeque::new())),
            thread_handle: None,
            thread_dead: Arc::new(AtomicBool::new(true)),
        }
    }

    ///
    /// Pops a value from the outqueue and returns Some(x) if it works, or None if
    /// it isn't successful
    /// *Note:
    ///     Internally locks the outqueue and then pops from the front (pop_front())
    /// 
    pub fn pop(&mut self) -> Option<Output> {
        self.outqueue.lock().unwrap().pop_front()
    }

    /// 
    /// Fills a buffer with data
    /// Internally uses the pop function repeatedly and leaves the value untouched if
    /// it can't pop a value out. 
    /// 
    /// Returns:
    ///     The number of successful pops in a usize
    /// 
    pub fn pop_vec(&mut self, buffer: &mut Vec<Output>) -> usize{
        let mut num_successful = 0;
        for i in 0..buffer.len(){
            if let Some(data) = self.pop(){
                buffer[i] = data;
                num_successful += 1;
            }
        }
        num_successful
    }

    ///
    /// Creates a thread and sets the appropriate flags/values to indicate that
    /// 
    fn create_thread(&mut self) {
        let inqueue_clone = self.inqueue.clone();
        let outqueue_clone = self.outqueue.clone();
        let thread_dead_clone = self.thread_dead.clone();
        let func_clone = self.function.clone();

        self.thread_dead.store(false, Ordering::Release);

        self.thread_handle = Some(thread::spawn(move || {
            while let Some(data) = inqueue_clone.lock().unwrap().pop_front() {
                outqueue_clone.lock().unwrap().push_back(func_clone(data));
            }

            thread_dead_clone.store(true, Ordering::Release);
        }));
    }

    ///
    /// Checks if the thread is dead and spawns one if it is
    /// 
    fn spawn_thread_if_dead(&mut self) {
        if self.thread_dead.load(Ordering::Acquire) {
            self.create_thread();
        }
    }

    /// 
    /// Locks the inqueue and pushes a value to the back and 
    /// creats a thread if needed
    /// 
    /// Parameters:
    ///     value:
    ///         A value of type Input to be pushed into the queue
    /// 
    pub fn enque(&mut self, value: Input) {
        self.inqueue.lock().unwrap().push_back(value);
        self.spawn_thread_if_dead();
    }

    ///
    /// Same as enque but instead pushes a Vec<Input> of values rather
    /// than a single one by iterating over the Vec and queueing it in
    /// the same way as enque() and then creats a thread if needed
    /// 
    /// Parameters:
    ///     values:
    ///         A Vec<Input> to be queued in a first-come first-serve fashion
    /// 
    pub fn enque_vec(&mut self, values: Vec<Input>) {
        for i in values {
            self.inqueue.lock().unwrap().push_back(i);
        }
        self.spawn_thread_if_dead();
    }

    ///
    /// Blocks the caller until the current thread is done (ie. It finishes all
    /// the leftover data in the queue as if it were on the same thread as the
    /// caller)
    /// 
    pub fn join(&mut self) {
        if !self.thread_dead.load(Ordering::Acquire) {
            let mut x = None;
            std::mem::swap(&mut x, &mut self.thread_handle);
            if let Some(y) = x {
                y.join().unwrap();
            }
        }
    }
}
