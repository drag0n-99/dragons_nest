pub struct Server {
    addr: String,
}

impl Server {
    pub fn new(addr: String) -> Self {
        // if the value of the struct property and the value of the 'value' for
        // that property have the same name then you can omit the 'value' for
        // the property name
        // Server { addr: addr }  becomes Server { addr }
        Server { addr }
    }

    pub fn run(self) {
        println!("Listening on {}", self.addr);
    }
}
