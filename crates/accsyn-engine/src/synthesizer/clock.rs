pub struct Clock {
    counter: u8,
}

impl Clock {
    pub fn new() -> Self {
        Self {
            counter: 0,
        }
    }

    pub fn tick_is_32nd_note(&mut self) -> bool {
        self.counter += 1;

        if self.counter == 2 {
            self.counter = 0;
            true
        } else {
            false
        }
    }

}