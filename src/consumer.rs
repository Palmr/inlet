use crate::array_string::ArrayString;
use crate::inlet::{ClientMeta, Inlet};

pub struct Consumer<EntryType, const ENTRY_COUNT: usize, const MAX_CONSUMERS: usize> {
    inlet: *mut Inlet<EntryType, ENTRY_COUNT, MAX_CONSUMERS>,
    consumer_index: usize,
}

impl<EntryType, const ENTRY_COUNT: usize, const MAX_CONSUMERS: usize>
    Consumer<EntryType, ENTRY_COUNT, MAX_CONSUMERS>
{
    pub fn new<I: Into<ArrayString>>(topic: I, consumer_id: I) -> Self {
        let inlet = Inlet::construct(topic);
        Self {
            inlet,
            consumer_index: Self::claim_consumer_entry(inlet, consumer_id),
        }
    }

    fn claim_consumer_entry<I: Into<ArrayString>>(
        wormhole: *mut Inlet<EntryType, ENTRY_COUNT, MAX_CONSUMERS>,
        name: I,
    ) -> usize {
        let name = name.into();
        if let Some((index, _consumer)) = unsafe { &mut *wormhole }
            .consumers
            .iter()
            .enumerate()
            .find(|(_i, c)| c.id == name)
        {
            index
        } else {
            let (index, consumer) = unsafe { &mut *wormhole }
                .consumers
                .iter_mut()
                .enumerate()
                .find(|(_index, consumer)| consumer.id.is_empty())
                .expect("Could not find an empty consumer to claim");
            consumer.id = name;
            index
        }
    }

    fn get_consumer(&self) -> &ClientMeta {
        &unsafe { &*self.inlet }.consumers[self.consumer_index]
    }

    fn get_consumer_mut(&mut self) -> &mut ClientMeta {
        &mut unsafe { &mut *self.inlet }.consumers[self.consumer_index]
    }

    pub fn has_data_to_consume(&self) -> bool {
        unsafe { &*self.inlet }.producer.sequence > self.get_consumer().sequence
    }

    pub fn process_current_entry<F: FnOnce(&EntryType)>(&mut self, handler: F) {
        let wormhole = unsafe { &*self.inlet };
        let sequence = self.get_consumer().sequence % ENTRY_COUNT;
        (handler)(&wormhole.data[sequence]);
        self.get_consumer_mut().sequence += 1;
    }

    pub fn process_entries<F: Fn(&EntryType)>(&mut self, handler: F) {
        loop {
            while !self.has_data_to_consume() {}

            self.process_current_entry(&handler);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::consumer::Consumer;
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
        cleanup("abc1");
        let mut producer = Producer::<TestEntry, 8, 2>::new(String::from("abc1"));
        let consumer =
            Consumer::<TestEntry, 8, 2>::new(String::from("abc1"), String::from("subscriber1"));
        assert!(!consumer.has_data_to_consume());
        producer.publish(|x| {
            x.value = 69420;
            x.value2 = 0xDEADBEEF
        });
        assert!(consumer.has_data_to_consume());
    }

    #[test]
    fn test_read_entry() {
        cleanup("abc2");
        let mut publisher = Producer::<TestEntry, 8, 2>::new(String::from("abc2"));
        let mut subscriber =
            Consumer::<TestEntry, 8, 2>::new(String::from("abc2"), String::from("subscriber1"));
        publisher.publish(|x| {
            x.value = 69420;
            x.value2 = 0xDEADBEEF
        });
        let mut value = 0;
        let mut value2 = 0;
        subscriber.process_current_entry(|x| {
            value = x.value;
            value2 = x.value2;
        });
        assert_eq!(value, 69420);
        assert_eq!(value2, 0xDEADBEEF);
    }
}
