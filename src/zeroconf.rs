// announce tcp_json address via dns-sd
use std::thread;
use std::time::Duration;
use astro_dnssd::register::DNSServiceBuilder;

pub fn start(port: u16) {
    thread::spawn(move || {
        let mut service = DNSServiceBuilder::new("_geckoaudio._tcp")
            .with_port(port)
            .with_name("Gecko Audio Streaming")
            .build()
            .unwrap();
        service.register(|_reply| ());
        loop {
            service.process_result();
        }
    });
}