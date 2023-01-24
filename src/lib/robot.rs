use core::ptr;
use cortex_m::interrupt::free;
use stm32f4::stm32f401::TIM3;

use crate::timers;

pub static mut INFRARED: [u16; 2] = [0, 0];

type Cm = u16;

#[derive(Default, Debug)]
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
    fn new(max_duty: *const u16, fd_duty: *mut u16, bk_duty: *mut u16) -> Motor {
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

impl Default for Robot {
    fn default() -> Self {
        Self {
            left_motor: get_left_motor(),
            right_motor: get_right_motor(),
            sensors: Default::default(),
        }
    }
}

impl Robot {
    pub fn new(left_motor: Motor, right_motor: Motor) -> Self {
        Self {
            sensors: Default::default(),
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

    pub fn lock_left_motor(&mut self, ticks: u32) {
        free(|cs| {
            let some_tim2 = timers::G_TIM2.borrow(cs).borrow();
            let tim2 = some_tim2.as_ref().unwrap();
            tim2.cnt.reset();
            tim2.arr.write(|w| w.arr().bits(ticks));
            tim2.cr1.modify(|_, w| w.cen().enabled());
        });
    }

    pub fn lock_right_motor(&mut self, ticks: u32) {
        free(|cs| {
            let some_tim5 = timers::G_TIM5.borrow(cs).borrow();
            let tim5 = some_tim5.as_ref().unwrap();
            tim5.cnt.reset();
            tim5.arr.write(|w| w.arr().bits(ticks));
            tim5.cr1.modify(|_, w| w.cen().enabled());
        });
    }
}

pub fn get_left_motor() -> Motor {
    let tim3 = unsafe { &*TIM3::PTR };
    Motor::new(
        tim3.arr.as_ptr() as *const u16,
        tim3.ccr3().as_ptr() as *mut u16,
        tim3.ccr4().as_ptr() as *mut u16,
    )
}

pub fn get_right_motor() -> Motor {
    let tim3 = unsafe { &*TIM3::PTR };
    Motor::new(
        tim3.arr.as_ptr() as *const u16,
        tim3.ccr1().as_ptr() as *mut u16,
        tim3.ccr2().as_ptr() as *mut u16,
    )
}
