use core::ptr;

type Cm = u16;

#[derive(Default)]
pub struct SensorReadings {
    pub front_distance: Cm,
    pub left_distance: Cm,
    pub left_infrared: u16,
    pub right_infrared: u16,
}

pub struct Motor {
    max_duty: *const u16,
    fd_duty: *mut u16,
    bk_duty: *mut u16,
}

#[derive(Clone, Copy, Debug)]
pub enum Dir {
    Fd,
    Bk,
}

impl Motor {
    pub unsafe fn new(max_duty: *const u16, fd_duty: *mut u16, bk_duty: *mut u16) -> Motor {
        unsafe {
            ptr::write_volatile(fd_duty, 0);
            ptr::write_volatile(bk_duty, 0);
        }
        Self {
            max_duty,
            fd_duty,
            bk_duty,
        }
    }

    pub fn forward(&mut self, duty: u16) {
        unsafe {
            ptr::write_volatile(self.fd_duty, duty.min(ptr::read_volatile(self.max_duty)));
            ptr::write_volatile(self.bk_duty, 0);
        }
    }

    pub fn backward(&mut self, duty: u16) {
        unsafe {
            ptr::write_volatile(self.bk_duty, duty.min(ptr::read_volatile(self.max_duty)));
            ptr::write_volatile(self.fd_duty, 0);
        }
    }

    pub fn stop(&mut self) {
        self.forward(0);
    }

    pub fn get_info(&self) -> (u16, Dir) {
        unsafe {
            let fd_duty = ptr::read_volatile(self.fd_duty);
            let bk_duty = ptr::read_volatile(self.bk_duty);
            if bk_duty == 0 {
                (fd_duty, Dir::Fd)
            } else {
                (bk_duty, Dir::Bk)
            }
        }
    }

    pub fn get_max_duty(&self) -> u16 {
        unsafe { ptr::read_volatile(self.max_duty) }
    }
}

pub struct Robot {
    sensors: SensorReadings,
    left_motor: Motor,
    right_motor: Motor,
}

impl Robot {
    pub fn new(left_motor: Motor, right_motor: Motor) -> Self {
        Self {
            sensors: SensorReadings::default(),
            left_motor,
            right_motor,
        }
    }

    pub fn update_sensors(&mut self, sr: SensorReadings) {
        self.sensors = sr;
    }

    pub fn get_sensor_readings(&self) -> &SensorReadings {
        &self.sensors
    }

    pub fn left_motor(&mut self) -> &mut Motor {
        &mut self.left_motor
    }

    pub fn right_motor(&mut self) -> &mut Motor {
        &mut self.right_motor
    }
}

unsafe impl Send for Robot {}
