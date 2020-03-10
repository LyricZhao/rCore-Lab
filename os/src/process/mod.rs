pub mod processor;
pub mod scheduler;
pub mod structs;
pub mod thread_pool;
pub mod timer;

use crate::fs::{INodeExt, ROOT_INODE};
use alloc::boxed::Box;
use processor::Processor;
use scheduler::RRScheduler;
use structs::Thread;
use thread_pool::ThreadPool;
use self::timer::Timer;
use crate::timer::now;
use lazy_static::lazy_static;
use spin::Mutex;

pub type Tid = usize;
pub type ExitCode = usize;

static CPU: Processor = Processor::new();

pub fn init() {
    let scheduler = RRScheduler::new(1);
    let thread_pool = ThreadPool::new(100, Box::new(scheduler));
    let idle = Thread::new_kernel(Processor::idle_main as usize);
    idle.append_initial_arguments([&CPU as *const Processor as usize, 0, 0]);
    CPU.init(idle, Box::new(thread_pool));

    execute("rust/user_shell", None);

    println!("++++ setup process!   ++++");
}

pub fn execute(path: &str, host_tid: Option<Tid>) -> bool {
    let find_result = ROOT_INODE.lookup(path);
    match find_result {
        Ok(inode) => {
            let data = inode.read_as_vec().unwrap();
            let user_thread = unsafe { Thread::new_user(data.as_slice(), host_tid) };
            CPU.add_thread(user_thread);
            true
        }
        Err(_) => {
            println!("command not found!");
            false
        }
    }
}

pub fn park() {
    CPU.park();
}

pub fn run() {
    CPU.run();
}

pub fn exit(code: usize) {
    CPU.exit(code);
}

pub fn yield_now() {
    CPU.yield_now();
}

pub fn wake_up(tid: Tid) {
    CPU.wake_up(tid);
}
pub fn current_tid() -> usize {
    CPU.current_tid()
}

pub fn current_thread_mut() -> &'static mut Thread {
    CPU.current_thread_mut()
}

lazy_static! {
    static ref TIMER: Mutex<Timer> = Mutex::new(Timer::default());
}

pub fn tick() {
    CPU.tick();
    TIMER.lock().tick(now());
}

pub fn sleep(sec: usize) {
    let tid = current_tid();
    TIMER
        .lock()
        .add(now() + (sec * 100) as u64, move || wake_up(tid));
    park();
}

/// Spawn a new kernel thread from function `f`.
pub fn spawn<F>(f: F)
    where
        F: FnOnce() + Send + 'static,
{
    let f = Box::into_raw(Box::new(f));
    let new_thread = Thread::new_kernel(entry::<F> as usize);
    new_thread.append_initial_arguments([f as usize, 0, 0]);
    CPU.add_thread(new_thread);

    // define a normal function, pass the function object from argument
    extern "C" fn entry<F>(f: usize) -> !
        where
            F: FnOnce() + Send + 'static,
    {
        let f = unsafe { Box::from_raw(f as *mut F) };
        f();
        exit(0);
        unreachable!()
    }
}