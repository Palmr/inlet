struct TestEntry {
    value: u64,
}

fn main() {
    let mut consumer = inlet::consumer::Consumer::<TestEntry, 8, 2>::new(
        String::from("example"),
        String::from("consumer1"),
    );

    consumer.process_entries(|e| println!("Value: {}", e.value));
}
