use {
    super::*,
    alloc::{collections::VecDeque, sync::Arc},
    spin::Mutex,
};

#[derive(Default)]
pub struct FifoPageReplace {
    frames: VecDeque<(usize, Arc<Mutex<PageTableImpl>>)>,
}

impl PageReplace for FifoPageReplace {
    fn push_frame(&mut self, vaddr: usize, pt: Arc<Mutex<PageTableImpl>>) {
        println!("push vaddr: {:#x?}", vaddr);
        self.frames.push_back((vaddr, pt));
    }

    fn choose_victim(&mut self) -> Option<(usize, Arc<Mutex<PageTableImpl>>)> {
        // 选择一个已经分配的物理页帧
        self.frames.pop_front()
    }

    fn tick(&self) {}
}

#[derive(Default)]
pub struct ClockPageReplace {
    frames: VecDeque<(usize, Arc<Mutex<PageTableImpl>>)>,
    current: usize
}

impl PageReplace for ClockPageReplace {
    fn push_frame(&mut self, vaddr: usize, pt: Arc<Mutex<PageTableImpl>>) {
        println!("push vaddr: {:#x?}", vaddr);
        self.frames.push_back((vaddr, pt));
    }

    fn choose_victim(&mut self) -> Option<(usize, Arc<Mutex<PageTableImpl>>)> {
        let mut i = 0;
        loop {
            let index = (i + self.current) % self.frames.len();
            let (vaddr, pt) = self.frames.get(index).as_mut().unwrap();
            if pt.lock().get_entry(*vaddr).unwrap().accessed() {
                pt.lock().get_entry(*vaddr).unwrap().clear_accessed();
            } else {
                let ret = self.frames.remove(index);
                self.current = if self.frames.len() > 0 {
                    index % self.frames.len()
                } else {
                    0
                };
                return ret;
            }
            i += 1;
        }
        unreachable!()
    }

    fn tick(&self) {}
}
