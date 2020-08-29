// announce tcp_json address via dns-sd
use std::thread;
use std::time::Duration;
use dns_sd::DNSService;

pub fn start(port: u16) {
    thread::spawn(move || {
        println!("Starting dns-sd service announce on port: {} ...", port);
        DNSService::register(Some("Gecko Audio Streaming"),
                                   "_geckoaudio._tcp",
                                   None,
                                   None,
                                   port,
                                   &[""]).unwrap();

        loop {
            thread::sleep(Duration::from_secs(10));
        }
    });
}