use csv::ReaderBuilder;
use std::error::Error;
use std::fs::File;
use std::collections::HashSet;

#[derive(Debug)]
pub struct FlowLabel {
    flows: HashSet<String>,
}

impl FlowLabel {
    pub fn new(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = ReaderBuilder::new().has_headers(false).from_reader(file);
        let mut flows = HashSet::new();

        for result in rdr.records() {
            let record = result?;
            let flow_str = record.iter().collect::<Vec<&str>>().join(", ");
            flows.insert(flow_str);
        }

        Ok(FlowLabel{flows})
    }

    pub fn is_tor(
		&self, src_ip: &str, dst_ip: &str, src_port: &str, dst_port: &str
	) -> bool {
        let flow_key_upstream = format!(
            "{}, {}, {}, {}", src_ip, dst_ip, src_port, dst_port
        );
        let flow_key_downstream = format!(
            "{}, {}, {}, {}", dst_ip, src_ip, dst_port, src_port
        );

        if self.flows.contains(
			&flow_key_upstream
		) || self.flows.contains(
			&flow_key_downstream
		) {
			true
		} else {
			false
		}
    }
}
