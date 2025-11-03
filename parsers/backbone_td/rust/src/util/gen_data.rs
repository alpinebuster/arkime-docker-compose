use rand::Rng;

pub fn random_flow(&previous_timestamp: &u64) -> (Vec<u8>, u64) {
    let mut rng = rand::thread_rng();
    
    // IP traces
    let flow_key: Vec<u8> = (0..5).map(|_| rng.gen_range(0..=255)).collect();
    let timestamp = previous_timestamp + rng.gen_range(1..=1000);

    (flow_key, timestamp)
}
