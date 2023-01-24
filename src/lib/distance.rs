use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
pub struct DistanceMeasurer {
    rising: Option<u16>,
    time_us: u16,
}

impl DistanceMeasurer {
    pub fn get_distance_cm(&self) -> u16 {
        self.time_us / 58
    }

    /// The t should be given in microseconds
    pub fn update_measurment(&mut self, t: u16) {
        self.rising = match self.rising {
            None => Some(t),
            Some(p) => {
                self.time_us = t.wrapping_sub(p);
                None
            }
        }
    }

    pub const fn new() -> Self {
        Self {
            rising: None,
            time_us: u16::MAX,
        }
    }
}

pub struct Measurements {
    pub front: DistanceMeasurer,
    pub side: DistanceMeasurer,
}

pub static G_DISTANCES: Mutex<RefCell<Measurements>> = Mutex::new(RefCell::new(Measurements {
    front: DistanceMeasurer::new(),
    side: DistanceMeasurer::new(),
}));
