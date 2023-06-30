use crate::array_string::ArrayString;
use crate::inlet::Inlet;
use std::ptr::{addr_of_mut, write_volatile};

pub struct Producer<EntryType, const ENTRY_COUNT: usize, const MAX_CONSUMERS: usize> {
    inlet: *mut Inlet<EntryType, ENTRY_COUNT, MAX_CONSUMERS>,
}

impl<EntryType, const ENTRY_COUNT: usize, const MAX_CONSUMERS: usize>
    Producer<EntryType, ENTRY_COUNT, MAX_CONSUMERS>
{
    pub fn new<I: Into<ArrayString>>(topic: I) -> Self {
        let inlet = Inlet::construct(topic);
        Self { inlet }
    }

    fn get_next_publisher_sequence(&self) -> usize {
        unsafe { &*self.inlet }.producer.sequence
    }

    fn get_minimum_consumer_sequence(&self) -> usize {
        unsafe { &*self.inlet }
            .consumers
            .iter()
            .filter(|c| !c.id.is_empty()) // todo - filter on timestamp
            .map(|c| c.sequence)
            .min()
            .unwrap_or(0)
    }

    pub fn publish<F: FnOnce(&mut EntryType)>(&mut self, f: F) {
        let next_sequence = self.get_next_publisher_sequence();
        while (next_sequence - self.get_minimum_consumer_sequence()) >= ENTRY_COUNT {
            println!(
                "spinning! {} {}",
                next_sequence,
                self.get_minimum_consumer_sequence()
            );
        }
        (f)(&mut unsafe { &mut *self.inlet }.data[next_sequence % ENTRY_COUNT]);
        let addr = addr_of_mut!(unsafe { &mut *self.inlet }.producer.sequence);
        unsafe { write_volatile(addr, (*self.inlet).producer.sequence + 1) };
    }
}

#[cfg(test)]
mod tests {
    use crate::producer::Producer;
    use std::fs;

    struct TestEntry {
        value: u64,
        value2: u64,
    }

    fn cleanup(topic: &str) {
        fs::remove_file(format!("inlet-{}", topic)).ok();
    }

    #[test]
    fn test_publish_entry() {
        cleanup("abc");
        let mut publisher = Producer::<TestEntry, 8, 2>::new(String::from("abc"));
        publisher.publish(|x| {
            x.value2 = 69;
            x.value = 0xDEADBEEF
        });
        publisher.publish(|x| {
            x.value2 = 70;
            x.value = 0xDEADBEEF
        });
        publisher.publish(|x| {
            x.value2 = 71;
            x.value = 0xDEADBEEF
        });
        publisher.publish(|x| {
            x.value2 = 72;
            x.value = 0xDEADBEEF
        });
        publisher.publish(|x| {
            x.value2 = 73;
            x.value = 0xDEADBEEF
        });
        publisher.publish(|x| {
            x.value2 = 74;
            x.value = 0xDEADBEEF
        });
        publisher.publish(|x| {
            x.value2 = 75;
            x.value = 0xDEADBEEF
        });
        publisher.publish(|x| {
            x.value2 = 76;
            x.value = 0xDEADBEEF
        });
    }
}
