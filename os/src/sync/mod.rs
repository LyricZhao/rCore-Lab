pub mod condvar;

pub use self::mutex::{Mutex as SleepLock, MutexGuard as SleepLockGuard};
mod mutex;