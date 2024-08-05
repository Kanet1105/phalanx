pub fn is_power_of_two(size: u32) -> bool {
    if size == 0 {
        return false;
    }

    (size & (size - 1)) == 0
}
