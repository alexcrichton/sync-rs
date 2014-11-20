use std::task::failing;

pub struct Flag { pub failed: bool }

impl Flag {
    pub fn borrow(&mut self) -> Guard {
        Guard { flag: &mut self.failed, failing: failing() }
    }
}

pub struct Guard<'a> {
    flag: &'a mut bool,
    failing: bool,
}

impl<'a> Guard<'a> {
    pub fn check(&self, name: &str) {
        if *self.flag {
            panic!("poisoned {} - another task failed inside", name);
        }
    }

    pub fn done(&mut self) {
        if !self.failing && failing() {
            *self.flag = true;
        }
    }
}
