use super::robot::Robot;
use cortex_m::asm;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum State {
    FollowingLine,
    FollowingLineAndAvoiding,
    Forward,
    TurningLeft,
    TurningRight,
    ReturnToLine,
    Avoiding,
    Stopped,
}

impl State {
    pub fn process_state(self, robot: &mut Robot) -> Self {
        match self {
            State::FollowingLine => following_line(robot),
            State::Stopped => State::Stopped,
            State::FollowingLineAndAvoiding => following_line_and_avoiding(robot),
            State::TurningRight => turning_right(robot),
            State::TurningLeft => turning_left(robot),
            State::Avoiding => avoiding(robot),
            State::Forward => forward(robot),
            Self::ReturnToLine => return_to_line(robot),
        }
    }
}

fn following_line(robot: &mut Robot) -> State {
    let readings = robot.get_sensor_readings();
    let left_is_black = readings.left_infrared > 300;
    let right_is_black = readings.right_infrared > 300;
    match (left_is_black, right_is_black) {
        (true, false) => {
            robot.left_motor().backward(u16::MAX);
            robot.right_motor().forward(u16::MAX);
        }
        (false, true) => {
            robot.left_motor().forward(u16::MAX);
            robot.right_motor().backward(u16::MAX);
        }
        _ => {
            robot.left_motor().forward(u16::MAX);
            robot.right_motor().forward(u16::MAX);
        }
    }
    State::FollowingLine
}

fn following_line_and_avoiding(robot: &mut Robot) -> State {
    if robot.get_sensor_readings().front_distance > 20 {
        following_line(robot);
        State::FollowingLineAndAvoiding
    } else {
        State::TurningRight
    }
}

fn turning_right(robot: &mut Robot) -> State {
    robot.left_motor().forward(u16::MAX);
    robot.right_motor().stop();
    robot.lock_left_motor(17);

    wait_till_stopped(robot);

    State::Avoiding
}

fn turning_left(robot: &mut Robot) -> State {
    robot.left_motor().backward(60_000);
    robot.right_motor().forward(43_000);
    robot.lock_left_motor(8);
    robot.lock_right_motor(8);

    wait_till_stopped(robot);

    State::Forward
}

fn avoiding(robot: &mut Robot) -> State {
    let sr = robot.get_sensor_readings();
    if sr.left_infrared > 400 || sr.right_infrared > 400 {
        robot.left_motor().backward(60_000);
        robot.right_motor().backward(46_000);
        robot.lock_left_motor(4);
        robot.lock_right_motor(4);
        wait_till_stopped(robot);
        State::ReturnToLine
    } else if sr.left_distance > 60 {
        robot.left_motor().forward(60_000);
        robot.right_motor().forward(46_000);
        robot.lock_left_motor(16);
        robot.lock_right_motor(16);
        wait_till_stopped(robot);
        State::TurningLeft
    } else if sr.front_distance < 20 {
        State::TurningRight
    } else {
        robot.left_motor().forward(60_000);
        robot.right_motor().forward(46_000);
        State::Avoiding
    }
}

fn return_to_line(robot: &mut Robot) -> State {
    if robot.get_sensor_readings().left_infrared > 1300 {
        robot.left_motor().forward(50_000);
        robot.lock_left_motor(4);
        wait_till_stopped(robot);
        return State::FollowingLineAndAvoiding;
    }
    robot.left_motor().forward(45_000);
    robot.lock_left_motor(2);
    wait_till_stopped(robot);
    State::ReturnToLine
}

fn forward(robot: &mut Robot) -> State {
    robot.left_motor().forward(60_000);
    robot.right_motor().forward(46_000);
    if robot.get_sensor_readings().left_distance > 30 {
        State::Forward
    } else {
        State::Avoiding
    }
}

fn wait_till_stopped(robot: &mut Robot) {
    while robot.left_motor().get_info().0 != 0 || robot.right_motor().get_info().0 != 0 {
        asm::nop();
    }
}
