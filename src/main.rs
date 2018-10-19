#[macro_use]
extern crate clap;
extern crate kafka;

use clap::{Arg, App};
use kafka::client::{FetchOffset, FetchPartition, KafkaClient};
use std::io;
use std::io::prelude::*;

fn main() {
    let args = App::new("kcat")
        .version(crate_version!())
        .about("cat for Apache Kafka")
        .arg(Arg::with_name("broker")
            .short("b")
            .long("broker")
            .value_name("BROKER")
            .help("Kafka broker")
            .takes_value(true))
        .arg(Arg::with_name("topic")
            .short("t")
            .long("topic")
            .value_name("TOPIC")
            .help("Kafka topic")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("partition")
            .short("p")
            .long("partition")
            .value_name("PARTITION")
            .help("Kafka topic partition")
            .takes_value(true)
            .default_value("0"))
        .get_matches();
    let broker = args.value_of("broker").unwrap_or("localhost:9092");
    let topic = args.value_of("topic").unwrap();
    let partition = value_t!(args, "partition", u32).unwrap_or(0);
    let mut client = KafkaClient::new(vec![broker.to_owned()]);
    let meta_res = client.load_metadata_all();
    if let Some(err) = meta_res.err() {
        println!("Error fetching metadata: {}", err);
        return;
    }

    let offsets = client.fetch_topic_offsets(topic, FetchOffset::Earliest).unwrap();
    let partiton_offset = offsets.get(partition as usize)
        .expect("failed to get offset for partition");
    let req = &[FetchPartition::new(topic, partition as i32, partiton_offset.offset)];
    let res = client.fetch_messages(req);
    match res {
        Ok(resps) => {
            let stdout = io::stdout();
            let mut writer = io::BufWriter::new(stdout.lock());
            for resp in resps {
                for t in resp.topics() {
                    for p in t.partitions() {
                        match p.data() {
                            &Ok(ref data) => {
                                for msg in data.messages() {
                                    let value_as_str = String::from_utf8_lossy(msg.value);
                                    let _ = writeln!(writer, "{}", &value_as_str);
                                }
                            }
                            &Err(ref e) => {
                                let _ = writeln!(&mut io::stderr(), "Partition error: {}:{}: {}", t.topic(), p.partition(), e).unwrap();
                            }
                        }
                    }
                }
            }
        }
        Err(err) => writeln!(&mut io::stderr(), "Error fetching messages: {}", err).unwrap(),
    }

}
