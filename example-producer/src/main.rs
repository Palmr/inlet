use std::time::Duration;

struct TestEntry {
    value: u64,
}

fn main() {
    let mut publisher = inlet::producer::Producer::<TestEntry, 8, 2>::new(String::from("example"));

    let mut counter: u64 = 0;
    loop {
        publisher.publish(|x| x.value = counter);
        counter += 1;
        std::thread::sleep(Duration::from_secs(1));
    }
}
