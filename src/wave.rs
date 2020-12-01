pub fn i16cos(x: i16) -> i16 {
    let mask = (x>>15) as i32;
    let t: i32 = ((x as i32)^mask) as i32;

    // There's an annoying overflow here, we need more safety range.
    let mut u = 1127;
    u = (u * t - 55399812) >> 15;
    u = (u * t + 1831554) >> 15;
    ((u * t + 8337525) >> 8) as i16
}

pub fn i16square(x: i16) -> i16 {
    if x < 0 {-0x7FFF} else {0x7FFF}
}

pub fn i16triangle(x: i16) -> i16 {
    (0x7FFFi16.wrapping_sub((if x < 0 {(-1i16).wrapping_sub(x)} else {x}) << 1)) as i16
}

pub fn i16saw(x: i16) -> i16 {
    x
}

