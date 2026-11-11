pub struct Serial {
    data: u8,
    control: u8,
    output: String,
}

impl Serial {
    pub fn new() -> Self {
        Self {
            data: 0,
            control: 0,
            output: String::new(),
        }
    }

    pub fn read_data(&self) -> u8 {
        self.data
    }

    pub fn write_data(&mut self, value: u8) {
        self.data = value;
    }

    pub fn read_control(&self) -> u8 {
        self.control
    }

    pub fn write_control(&mut self, value: u8) -> bool {
        self.control = value;

        if value == 0x81 {
            self.output.push(self.data as char);

            // Transfer completes immediately in our simplified model.
            self.control &= !0x80;

            return true;
        }

        false
    }

    pub fn output(&self) -> &str {
        &self.output
    }
}
