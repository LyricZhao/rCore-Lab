use super::Tid;
use alloc::vec::Vec;

pub trait Scheduler {
    fn push(&mut self, tid: Tid);
    fn pop(&mut self) -> Option<Tid>;
    fn tick(&mut self) -> bool;
    fn exit(&mut self, tid: Tid);
}

#[derive(Default)]
struct RRInfo {
    valid: bool,
    time: usize,
    prev: usize,
    next: usize,
}

pub struct RRScheduler {
    threads: Vec<RRInfo>,
    max_time: usize,
    current: usize,
}

impl RRScheduler {
    pub fn new(max_time_slice: usize) -> Self {
        let mut rr = RRScheduler {
            threads: Vec::default(),
            max_time: max_time_slice,
            current: 0,
        };
        rr.threads.push(RRInfo {
            valid: false,
            time: 0,
            prev: 0,
            next: 0,
        });
        rr
    }
}
impl Scheduler for RRScheduler {
    fn push(&mut self, tid: Tid) {
        let tid = tid + 1;
        if tid + 1 > self.threads.len() {
            self.threads.resize_with(tid + 1, Default::default);
        }

        if self.threads[tid].time == 0 {
            self.threads[tid].time = self.max_time;
        }

        let prev = self.threads[0].prev;
        self.threads[tid].valid = true;
        self.threads[prev].next = tid;
        self.threads[tid].prev = prev;
        self.threads[0].prev = tid;
        self.threads[tid].next = 0;
    }

    fn pop(&mut self) -> Option<Tid> {
        let ret = self.threads[0].next;
        if ret != 0 {
            let next = self.threads[ret].next;
            let prev = self.threads[ret].prev;
            self.threads[next].prev = prev;
            self.threads[prev].next = next;
            self.threads[ret].prev = 0;
            self.threads[ret].next = 0;
            self.threads[ret].valid = false;
            self.current = ret;
            Some(ret - 1)
        } else {
            None
        }
    }

    // 当前线程的可用时间片 -= 1
    fn tick(&mut self) -> bool {
        let tid = self.current;
        if tid != 0 {
            self.threads[tid].time -= 1;
            if self.threads[tid].time == 0 {
                return true;
            } else {
                return false;
            }
        }
        return true;
    }

    fn exit(&mut self, tid: Tid) {
        let tid = tid + 1;
        if self.current == tid {
            self.current = 0;
        }
    }
}

const BIG_STRIDE: usize = 40320; // 8!

struct SInfo {
    stride: usize,
    pass: usize,
    prev: usize,
    next: usize,
}

impl SInfo {
    fn add(&mut self) {
        self.stride += self.pass;
    }
}

impl Default for SInfo {
    fn default() -> SInfo {
        SInfo {
            stride: 0,
            pass: BIG_STRIDE / 1,
            prev: 0,
            next: 0,
        }
    }
}

pub struct StrideScheduler {
    threads: Vec<SInfo>,
    current: usize,
}

impl StrideScheduler {
    pub fn new() -> Self {
        let mut scheduler = StrideScheduler {
            threads: Vec::default(),
            current: 0,
        };
        scheduler.threads.push(SInfo::default());
        scheduler
    }

    pub fn set_priority(&mut self, priority: usize, tid: Tid) {
        self.threads[tid + 1].pass = BIG_STRIDE / priority;
    }

    fn resort(&mut self, tid: Tid) {
        let stride = self.threads[tid].stride;

        let mut prev = 0;
        let mut next = 0;
        loop {
            next = self.threads[prev].next;
            if next == 0 || self.threads[next].stride >= stride {
                break;
            }
            prev += 1;
        }

        self.threads[prev].next = tid;
        self.threads[tid].prev = prev;
        self.threads[tid].next = next;
        if next > 0 {
            self.threads[next].prev = tid;
        }
    }

    fn remove(&mut self, tid: Tid) {
        assert_ne!(tid, 0);
        let (prev, next) = (self.threads[tid].prev, self.threads[tid].next);
        self.threads[prev].next = next;
        if next > 0 {
            self.threads[next].prev = prev;
        }
    }
}

impl Scheduler for StrideScheduler {
    fn push(&mut self, tid: usize) {
        // Thread tid is not in the scheduling pool before push()
        let tid = tid + 1;
        if tid >= self.threads.len() {
            // Note that we have to use resize because index may bigger than length
            self.threads.resize_with(tid + 1, SInfo::default);
        }

        self.resort(tid);
    }

    fn pop(&mut self) -> Option<usize> {
        let index = self.threads[0].next;
        if index > 0 {
            self.remove(index);
            self.threads[index].add();
            self.current = index;
            Some(index - 1)
        } else {
            None
        }
    }

    fn tick(&mut self) -> bool {
        let tid = self.current;
        if tid > 0 {
            let next = self.threads[0].next;
            if next == 0 {
                return false;
            }

            return if self.threads[tid].stride <= self.threads[next].stride {
                self.threads[tid].add();
                false
            } else {
                true
            }
        }
        true
    }

    fn exit(&mut self, tid: usize) {
        let tid = tid + 1;
        self.current = 0;
        self.remove(tid);
        self.threads[tid] = SInfo::default();
    }
}
